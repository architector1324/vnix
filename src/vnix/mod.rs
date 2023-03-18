pub mod core;
pub mod serv;
pub mod utils;

use alloc::boxed::Box;

use crate::driver::{CLIErr, DrvErr};

use self::core::user::Usr;
use self::core::task::TaskRun;
use self::core::kern::{Kern, KernErr};
use self::core::serv::{Serv, ServHlr};
use self::core::unit::{Unit, UnitParse};

use self::serv::{io, sys, math, gfx, time, test};

pub fn vnix_entry(mut kern: Kern) -> Result<(), KernErr> {
    // register service
    let services = [
        // // ("io.term", Box::new(io::term::Term::default()) as Box<dyn ServHlr>),
        (io::term::SERV_PATH, io::term::SERV_HELP, Box::new(io::term::term_hlr) as Box<ServHlr>),
        (io::store::SERV_PATH, io::store::SERV_HELP, Box::new(io::store::store_hlr) as Box<ServHlr>),
        // // // ("etc.fsm", Box::new(etc::fsm::FSM::default()) as Box<dyn ServHlr>),
        (time::chrono::SERV_PATH, time::chrono::SERV_HELP, Box::new(time::chrono::chrono_hlr) as Box<ServHlr>),
        (gfx::gfx2d::SERV_PATH, gfx::gfx2d::SERV_HELP, Box::new(gfx::gfx2d::gfx2d_hlr) as Box<ServHlr>),
        (math::calc::SERV_PATH, math::calc::SERV_HELP, Box::new(math::calc::calc_hlr) as Box<ServHlr>),
        (sys::task::SERV_PATH, sys::task::SERV_HELP, Box::new(sys::task::task_hlr) as Box<ServHlr>),
        (sys::usr::SERV_PATH, sys::usr::SERV_HELP, Box::new(sys::usr::usr_hlr) as Box<ServHlr>),
        (sys::hw::SERV_PATH, sys::hw::SERV_HELP, Box::new(sys::hw::hw_hlr) as Box<ServHlr>),
        (test::dump::SERV_PATH, test::dump::SERV_HELP, Box::new(test::dump::dump_hlr) as Box<ServHlr>),
        (test::echo::SERV_PATH, test::echo::SERV_HELP, Box::new(test::echo::echo_hlr) as Box<ServHlr>),
    ];

    for (name, help, hlr) in services {
        let serv = Serv::new(name, help, hlr);
        kern.reg_serv(serv)?;

        writeln!(kern.drv.cli, "INFO vnix:kern: service `{}` registered", name).map_err(|_| KernErr::DrvErr(DrvErr::CLI(CLIErr::Write)))?;
    }

    // register user
    let _super = Usr::new("super", &mut kern)?.0;
    kern.reg_usr(_super.clone())?;

    writeln!(kern.drv.cli, "INFO vnix:kern: user `{}` registered", _super).map_err(|_| KernErr::DrvErr(DrvErr::CLI(CLIErr::Write)))?;

    // test
    // let s = "(load @task.test)@io.store";
    // let s = "{task.sim:[a@test.dump b@test.dump]}";
    // let s = "{task.que:[test@sys.usr a@test.dump b@test.dump]}";
    let s = "{fill:((320 240) #ff0000)}";
    let test_msg = Unit::parse(s.chars()).map_err(|e| KernErr::ParseErr(e))?.0;

    // run
    // let path = Unit::parse("@task.init.gfx.cli".chars()).map_err(|e| KernErr::ParseErr(e))?.0;
    // let msg = kern.ram_store.load(path).ok_or(KernErr::DbLoadFault)?;

    let run = TaskRun(test_msg, "gfx.2d".into());

    kern.reg_task(&_super.name, "init.load", run)?;

    kern.run()
}
