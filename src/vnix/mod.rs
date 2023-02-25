pub mod core;
pub mod serv;
pub mod utils;

use alloc::boxed::Box;
use alloc::vec;

use crate::driver::CLIErr;

use self::core::task::TaskLoop;
use self::core::unit::Unit;
use self::core::user::Usr;
use self::core::kern::{Kern, KernErr};
use self::core::serv::{Serv, ServHlr};
use self::serv::{io, math, gfx, etc, sys, test};


pub fn vnix_entry(mut kern: Kern) -> Result<(), KernErr> {
    // register service
    let services = [
        ("io.term", Box::new(io::term::Term::default()) as Box<dyn ServHlr>),
        ("io.store", Box::new(io::store::Store::default()) as Box<dyn ServHlr>),
        ("etc.chrono", Box::new(etc::chrono::Chrono::default()) as Box<dyn ServHlr>),
        ("etc.fsm", Box::new(etc::fsm::FSM::default()) as Box<dyn ServHlr>),
        ("gfx.2d", Box::new(gfx::GFX2D::default()) as Box<dyn ServHlr>),
        ("math.int", Box::new(math::Int::default()) as Box<dyn ServHlr>),
        ("sys.task", Box::new(sys::task::Task::default()) as Box<dyn ServHlr>),
        ("sys.usr", Box::new(sys::usr::User::default()) as Box<dyn ServHlr>),
        ("sys.hw", Box::new(sys::hw::HW::default()) as Box<dyn ServHlr>),
        ("test.dumb", Box::new(test::Dumb::default()) as Box<dyn ServHlr>),
    ];

    for (name, hlr) in services {
        let serv = Serv::new(name, hlr);
        kern.reg_serv(serv)?;

        writeln!(kern.drv.cli, "INFO vnix:kern: service `{}` registered", name).map_err(|_| KernErr::CLIErr(CLIErr::Write))?;
    }

    // register user
    let _super = Usr::new("super", &mut kern)?.0;
    kern.reg_usr(_super.clone())?;

    writeln!(kern.drv.cli, "INFO vnix:kern: user `{}` registered", _super).map_err(|_| KernErr::CLIErr(CLIErr::Write))?;

    // // test
    // let s = "[(set.res.gfx (1920 1080)) cls.gfx (load @img.wall.ai.1)@io.store]";
    // let msg = Unit::parse(s.chars()).map_err(|e| KernErr::ParseErr(e))?.0;

    // let task = TaskLoop::Queue(vec![(msg, "io.term".into())]);

    // run
    let path = Unit::parse("@task.init.gfx.cli".chars()).map_err(|e| KernErr::ParseErr(e))?.0;
    let msg = kern.ram_store.load(path).ok_or(KernErr::DbLoadFault)?;

    let task = TaskLoop::Queue(vec![(msg, "sys.task".into())]);
    kern.reg_task(&_super.name, "init.load", task)?;

    kern.run()
}
