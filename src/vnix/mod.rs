pub mod core;
pub mod serv;
pub mod utils;

use ::core::writeln;
use ::core::fmt::Write;

use crate::vnix::core::driver::{CLIErr, DrvErr};

use self::core::user::Usr;
use self::core::task::TaskRun;
use self::core::kern::{Kern, KernErr};
use self::core::serv::{Serv, ServHlr};
use self::core::unit::{Unit, UnitParse};

use self::serv::{io, sys, math, gfx, dat, time, test};


pub fn vnix_entry(mut kern: Kern) -> Result<(), KernErr> {
    // register service
    let services = [
        (io::term::SERV_PATH, io::term::help::SERV_HELP, Box::new(io::term::TermHlr) as Box<dyn ServHlr>),
        (io::store::SERV_PATH, io::store::SERV_HELP, Box::new(io::store::StoreHlr) as Box<dyn ServHlr>),
        // // ("auto.fsm", Box::new(etc::fsm::FSM::default()) as Box<dyn ServHlr>),
        (dat::proc::SERV_PATH, dat::proc::SERV_HELP, Box::new(dat::proc::ProcHlr) as Box<dyn ServHlr>),
        (dat::gen::SERV_PATH, dat::gen::SERV_HELP, Box::new(dat::gen::GenHlr) as Box<dyn ServHlr>),
        (time::chrono::SERV_PATH, time::chrono::SERV_HELP, Box::new(time::chrono::ChronoHlr) as Box<dyn ServHlr>),
        (gfx::gfx2d::SERV_PATH, gfx::gfx2d::SERV_HELP, Box::new(gfx::gfx2d::GFX2DHlr) as Box<dyn ServHlr>),
        (math::calc::SERV_PATH,  math::calc::SERV_HELP, Box::new(math::calc::CalcHlr) as Box<dyn ServHlr>),
        (sys::task::SERV_PATH, sys::task::SERV_HELP, Box::new(sys::task::TaskHlr) as Box<dyn ServHlr>),
        (sys::usr::SERV_PATH, sys::usr::SERV_HELP, Box::new(sys::usr::UsrHlr) as Box<dyn ServHlr>),
        (sys::hw::SERV_PATH, sys::hw::SERV_HELP, Box::new(sys::hw::HWHlr) as Box<dyn ServHlr>),
        (test::dump::SERV_PATH, test::dump::SERV_HELP, Box::new(test::dump::DumpHlr) as Box<dyn ServHlr>),
        (test::echo::SERV_PATH, test::echo::SERV_HELP, Box::new(test::echo::EchoHlr) as Box<dyn ServHlr>),
        (test::void::SERV_PATH, test::void::SERV_HELP, Box::new(test::void::VoidHlr) as Box<dyn ServHlr>)
    ];

    for (name, help, hlr) in services {
        let serv = Serv::new(name, help, hlr);
        kern.reg_serv(serv)?;

        writeln!(kern, "INFO vnix:kern: service `{}` registered", name).map_err(|_| KernErr::DrvErr(DrvErr::CLI(CLIErr::Write)))?;
    }

    // register user
    let _super = Usr::new("super", &mut kern)?.0;
    kern.reg_usr(_super.clone())?;

    writeln!(kern, "INFO vnix:kern: user `{}` registered", _super).map_err(|_| KernErr::DrvErr(DrvErr::CLI(CLIErr::Write)))?;

    // test
    // let s = "{
    //     task.stk:[
    //         (set.res.gfx '720p')
    //         cls
    //         (say `loading video..`)
    //         (load @vid.vnix.logo.720p)@io.store
    //         (set.res.gfx '1080p')
    //         cls
    //         (say `loading video..`)
    //         (load @vid.vnix.logo.1080p)@io.store
    //         (say done)
    //     ]@io.term   
    // }";
    // let s = "(task.stk [cls (load @vid.sonic)@io.store (say done)]@io.term)";
    // let s = "{say:{a:[1 {b:c} 3] d:-} nice:4 nl:t}@io.term";
    // let msg = Unit::parse(s.chars()).map_err(|e| KernErr::ParseErr(e))?.0;

    // let s = "123";
    // let msg = Unit::parse(s.chars()).map_err(|e| KernErr::ParseErr(e))?.0;

    // let run = TaskRun(msg, "io.term".into());
    // kern.reg_task(&_super.name, "test", run)?;

    // kern.run()

    // run
    let path = Unit::parse("@task.init".chars()).map_err(|e| KernErr::ParseErr(e))?.0;
    let msg = kern.ram_store.load(path).ok_or(KernErr::DbLoadFault)?;

    let run = TaskRun(msg, "sys.task".into());

    kern.reg_task(&_super.name, "init.load", run)?;

    kern.run()
}
