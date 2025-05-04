use std::{
    cell::RefCell,
    rc::Rc,
    time::{Duration, Instant},
};

use mlua::Lua;

use crate::{
    events::WireChangeEvent,
    module_id::PinAddress,
    parser::{self},
    pin_state::WireState,
    system::System,
};

fn load_execute(lua: &mut Lua, sys: Rc<RefCell<System>>) -> mlua::Result<()> {
    let execute_fn = lua.create_function(move |_, cycles: i64| {
        sys.borrow_mut().run_for(cycles);
        Ok(())
    })?;
    lua.globals().set("execute", execute_fn)
}

fn load_set_wire(lua: &mut Lua, sys: Rc<RefCell<System>>) -> mlua::Result<()> {
    let set_wire_fn = lua.create_function(move |_, (id, value): (String, bool)| {
        let receiver_id = sys.borrow().pin_address(&id);
        sys.borrow().system_tables.inbox.read().unwrap().send(
            WireChangeEvent {
                receiver_id,
                state: if value {
                    WireState::High
                } else {
                    WireState::Low
                },
            },
            sys.borrow().t,
        );
        Ok(())
    })?;
    lua.globals().set("set_wire", set_wire_fn)
}

fn load_set_wires(lua: &mut Lua, sys: Rc<RefCell<System>>) -> mlua::Result<()> {
    let set_wires_fn =
        lua.create_function(move |_, (comp, msb, lsb, value): (String, u8, u8, i64)| {
            let sys_ref = sys.borrow();
            let module = sys_ref.id_map.get(&comp).unwrap();
            let bits = (msb as i32 - lsb as i32 + 1).abs();
            let inbox = sys_ref.system_tables.inbox.read().unwrap();
            for i in 0..bits {
                let pin = if lsb < msb {
                    lsb + i as u8
                } else {
                    msb + i as u8
                };
                let receiver_id = module.with_pin(pin);

                inbox.send(
                    WireChangeEvent {
                        receiver_id,
                        state: if (value >> i) & 1 == 1 {
                            WireState::High
                        } else {
                            WireState::Low
                        },
                    },
                    sys.borrow().t,
                );
            }
            Ok(())
        })?;
    lua.globals().set("set_wires", set_wires_fn)
}

fn load_get_wire(lua: &mut Lua, sys: Rc<RefCell<System>>) -> mlua::Result<()> {
    let get_wire_fn = lua.create_function(move |_, id: String| {
        let pin_addr = sys.borrow().pin_address(&id);
        let state = sys.borrow().get_pin(pin_addr);
        Ok(state.to_bool())
    })?;
    lua.globals().set("get_wire", get_wire_fn)
}

fn load_get_wires(lua: &mut Lua, sys: Rc<RefCell<System>>) -> mlua::Result<()> {
    let get_wires_fn = lua.create_function(move |_, (comp, msb, lsb): (String, u8, u8)| {
        let sys_ref = sys.borrow();
        let module = sys_ref.id_map.get(&comp).unwrap();
        let bits = (msb as i32 - lsb as i32 + 1).abs();

        let mut value = 0;
        for i in 0..bits {
            let pin = if lsb < msb {
                lsb + i as u8
            } else {
                msb + i as u8
            };
            let pin_addr = module.with_pin(pin);
            let state = sys.borrow().get_pin(pin_addr);
            value |= (state.to_bool() as u64) << i;
        }
        Ok(value)
    })?;
    lua.globals().set("get_wires", get_wires_fn)
}

fn load_support_lib(lua: &mut Lua, sys: Rc<RefCell<System>>) -> mlua::Result<()> {
    load_execute(lua, sys.clone())?;
    load_set_wire(lua, sys.clone())?;
    load_get_wire(lua, sys.clone())?;
    load_set_wires(lua, sys.clone())?;
    load_get_wires(lua, sys.clone())?;
    Ok(())
}

pub enum TestResult {
    Success(Duration),
    #[allow(dead_code)]
    Failure(Vec<String>),
    Error(mlua::Error, Vec<String>),
}

pub fn run_test(sys_filename: &str, test_filename: &str, vcd_enabled: bool) -> TestResult {
    let sys: Rc<RefCell<System>> = Rc::new(RefCell::new(parser::load(sys_filename, vcd_enabled)));

    let mut lua = Lua::new();
    load_support_lib(&mut lua, sys.clone()).unwrap();
    let test_src = match std::fs::read_to_string(test_filename) {
        Ok(src) => src,
        Err(err) => return TestResult::Error(err.into(), Vec::new()),
    };
    let start = Instant::now();

    let result = lua.load(test_src).exec();
    let simulation_time = start.elapsed();
    match result {
        Ok(()) => TestResult::Success(simulation_time),
        Err(err) => TestResult::Error(
            err,
            sys.borrow().system_tables.messages.read().unwrap().clone(),
        ),
    }
}
