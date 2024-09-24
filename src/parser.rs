use std::collections::HashMap;

use yaml_rust2::{Yaml, YamlLoader};

use crate::{
    components::{avr::mcu, led::Led},
    events::EventQueue,
    module::ActiveModule,
    module_id::{ModuleAddress, PinAddress},
    wiring::{InboxTable, WiringTable},
};

fn parse_passive_component(
    parent: &mut dyn ActiveModule,
    component: &Yaml,
    parent_name: &str,
    id: &str,
    id_map: &mut HashMap<String, ModuleAddress>,
) {
    let addr = match component["type"].as_str().unwrap() {
        "led" => {
            let led = parent.module_store().add_module(|id| Led::new(id));
            led.address()
        }
        _ => unimplemented!(),
    };
    let name = format!("{}.{}", parent_name, id);

    id_map.insert(name, addr);
}

fn parse_active_component(
    root_prefix: u8,
    component: &Yaml,
    id: &str,
    it: &mut InboxTable,
    id_map: &mut HashMap<String, ModuleAddress>,
) -> Box<dyn ActiveModule> {
    let event_queue = EventQueue::new(1, root_prefix, it.add_listener(root_prefix));

    id_map.insert(id.to_string(), ModuleAddress::root().child_id(root_prefix));
    match component["type"].as_str().unwrap() {
        "mcu" => {
            let memory = component["memory"].as_str().unwrap();
            let mut mcu = mcu::Mcu::new(event_queue).with_flash_hex(memory);
            for (name, sub_component) in component["components"].as_hash().unwrap() {
                parse_passive_component(
                    &mut mcu,
                    sub_component,
                    id,
                    name.as_str().unwrap(),
                    id_map,
                );
            }
            Box::new(mcu)
        }
        _ => unimplemented!(),
    }
}

fn find_module_id(
    name: &str,
    id_map: &HashMap<String, ModuleAddress>,
    components: &[Box<dyn ActiveModule>],
) -> PinAddress {
    let (name, pin) = name.split_once(':').unwrap();
    let mut addr = *id_map.get(name).unwrap();
    let root = components[addr.current() as usize].as_ref();
    addr.advance();
    if addr.is_empty() {
        PinAddress::from(root, pin.parse::<u8>().unwrap())
    } else {
        let m = root.find(addr).unwrap();
        PinAddress::from(m, pin.parse::<u8>().unwrap())
    }
}

pub fn load(path: &str) -> Vec<Box<dyn ActiveModule>> {
    let yaml = std::fs::read_to_string(path).unwrap();
    let data = &YamlLoader::load_from_str(&yaml).unwrap()[0];

    let mut it = InboxTable::new();
    let mut wt = WiringTable::new();

    let mut id_map: HashMap<String, ModuleAddress> = HashMap::new();

    let mut components = vec![];

    let mut root_prefix = 0;
    for (id, component) in data["components"].as_hash().unwrap() {
        components.push(parse_active_component(
            root_prefix,
            component,
            id.as_str().unwrap(),
            &mut it,
            &mut id_map,
        ));
        root_prefix += 1;
    }

    for wire in data["wires"].as_vec().unwrap() {
        let from_name = wire["from"].as_str().unwrap();
        let to_name = wire["to"].as_str().unwrap();

        let from = find_module_id(from_name, &id_map, &components);
        let to = find_module_id(to_name, &id_map, &components);
        wt.add_wire(from, vec![to])
    }

    it.save();
    wt.save();

    components
}
