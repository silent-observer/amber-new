use std::{
    cell::RefCell,
    rc::Rc,
    time::{Duration, Instant},
};

use mlua::Lua;

use crate::{
    events::WireChangeEvent,
    module::ActiveModule,
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

fn load_get_wire(lua: &mut Lua, sys: Rc<RefCell<System>>) -> mlua::Result<()> {
    let get_wire_fn = lua.create_function(move |_, (id, value): (String, bool)| {
        let pin_addr = sys.borrow().pin_address(&id);
        let state = sys.borrow().get_pin(pin_addr);
        Ok(state.to_bool())
    })?;
    lua.globals().set("get_wire", get_wire_fn)
}

fn load_support_lib(lua: &mut Lua, sys: Rc<RefCell<System>>) -> mlua::Result<()> {
    load_execute(lua, sys.clone())?;
    load_set_wire(lua, sys.clone())?;
    load_get_wire(lua, sys.clone())?;
    Ok(())
}

pub fn run_test(sys_filename: &str, test_filename: &str) -> Result<Duration, mlua::Error> {
    let sys: Rc<RefCell<System>> = Rc::new(RefCell::new(parser::load(sys_filename)));

    let mut lua = Lua::new();
    load_support_lib(&mut lua, sys).unwrap();
    let test_src = std::fs::read_to_string(test_filename)?;
    let start = Instant::now();

    let result = lua.load(test_src).exec();
    let simulation_time = start.elapsed();
    match result {
        Ok(()) => Ok(simulation_time),
        Err(err) => Err(err),
    }
}
