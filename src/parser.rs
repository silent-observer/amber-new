use std::collections::HashMap;

use yaml_rust2::{Yaml, YamlLoader};

use crate::{
    components::{avr::mcu, led::Led},
    events::EventQueue,
    module::ActiveModule,
    module_id::ModuleAddress,
    system::{find_pin_addr, System},
    system_tables::SystemTables,
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

fn parse_active_component<'a>(
    root_prefix: u8,
    component: &Yaml,
    id: &str,
    system_tables: SystemTables,
    id_map: &mut HashMap<String, ModuleAddress>,
) -> Box<dyn ActiveModule + 'a> {
    let recv = system_tables
        .inbox
        .write()
        .unwrap()
        .add_listener(root_prefix);
    let event_queue = EventQueue::new(system_tables, 1, root_prefix, recv);

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

pub fn load(path: &str) -> System {
    let yaml = std::fs::read_to_string(path).unwrap();
    let data = &YamlLoader::load_from_str(&yaml).unwrap()[0];

    let system_tables = SystemTables::new();

    let mut id_map: HashMap<String, ModuleAddress> = HashMap::new();

    let mut components = vec![];

    let mut root_prefix = 0;
    for (id, component) in data["components"].as_hash().unwrap() {
        components.push(parse_active_component(
            root_prefix,
            component,
            id.as_str().unwrap(),
            system_tables.clone(),
            &mut id_map,
        ));
        root_prefix += 1;
    }

    for wire in data["wires"].as_vec().unwrap() {
        let from_name = wire["from"].as_str().unwrap();
        let to_name = wire["to"].as_str().unwrap();

        let from = find_pin_addr(from_name, &id_map, &components);
        let to = find_pin_addr(to_name, &id_map, &components);
        system_tables
            .wiring
            .write()
            .unwrap()
            .add_wire(from, vec![to])
    }

    System {
        system_tables,
        modules: components,
        id_map,
        t: 0,
    }
}
