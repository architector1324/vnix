use crate::{vnix::core::{serv::{ServHlr, Serv, ServHelpTopic, ServHlrAsync}, kern::{Kern, KernErr}, msg::Msg, unit::{FromUnit, Unit, SchemaUnit, Schema}}, driver::CLIErr};
use alloc::boxed::Box;
use alloc::string::String;
use spin::Mutex;


pub struct Dumb {
    msg: Option<Unit>
}

pub struct DumbLoop {
    msg: Option<Unit>
}

impl Default for Dumb {
    fn default() -> Self {
        Dumb{msg: None}
    }
}

impl Default for DumbLoop {
    fn default() -> Self {
        DumbLoop{msg: None}
    }
}

impl FromUnit for Dumb {
    fn from_unit_loc(u: &Unit) -> Option<Self> {
        let schm = SchemaUnit;
        schm.find_loc(u).map(|u| {
            Dumb{msg: Some(u)}
        })
    }
}

impl FromUnit for DumbLoop {
    fn from_unit_loc(u: &Unit) -> Option<Self> {
        let schm = SchemaUnit;
        schm.find_loc(u).map(|u| {
            DumbLoop{msg: Some(u)}
        })
    }
}


impl ServHlr for Dumb {
    fn help<'a>(self, _ath: String, _topic: ServHelpTopic, _kern: &'a Mutex<Kern>) -> ServHlrAsync<'a> {
        unimplemented!()
    }

    fn handle<'a>(self, _msg: Msg, _serv: Serv, kern: &'a Mutex<Kern>) -> ServHlrAsync<'a> {
        let hlr = move || {
            if let Some(msg) = self.msg {
                writeln!(kern.lock().drv.cli, "test: {msg}").map_err(|_| KernErr::CLIErr(CLIErr::Write))?;
                yield;
            }
            Ok(None)
        };
        ServHlrAsync(Box::new(hlr))
    }
}

impl ServHlr for DumbLoop {
    fn help<'a>(self, _ath: String, _topic: ServHelpTopic, _kern: &'a Mutex<Kern>) -> ServHlrAsync<'a> {
        unimplemented!()
    }

    fn handle<'a>(self, _msg: Msg, _serv: Serv, kern: &'a Mutex<Kern>) -> ServHlrAsync<'a> {
        let hlr = move || {
            if let Some(msg) = self.msg {
                for i in 0..5 {
                    writeln!(kern.lock().drv.cli, "test {i}: {msg}").map_err(|_| KernErr::CLIErr(CLIErr::Write))?;
                    yield;
                }
            }
            Ok(None)
        };
        ServHlrAsync(Box::new(hlr))
    }
}
