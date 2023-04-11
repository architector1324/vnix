use core::pin::Pin;
use core::fmt::{Display, Write};
use core::ops::{Generator, GeneratorState};

use alloc::vec;
use alloc::rc::Rc;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::string::String;
use num::{BigInt, BigRational};

use super::msg::Msg;
use super::user::Usr;
use super::task::{Task, TaskRun, TaskSig};
use super::unit::{Unit, UnitParseErr, UnitAs, UnitNew, Path, UnitBase, Int, Dec};
use super::serv::{Serv, ServErr, ServHlrAsync};
use super::driver::{CLIErr, CLI, Disp, Time, Rnd, Mem, DrvErr};

use crate::thread;

use crate::vnix::serv::io::term::base;
use crate::vnix::utils::{RamStore, Maybe};

use spin::Mutex;


#[derive(Debug, PartialEq, Clone)]
pub enum Addr {
    Local,
    Remote([u16; 8])
}

#[derive(Debug)]
pub enum KernErr {
    MemoryOut,
    EncodeFault,
    DecodeFault,
    CompressionFault,
    DecompressionFault,
    CreatePrivKeyFault,
    CreatePubKeyFault,
    SignFault,
    SignVerifyFault,
    HashVerifyFault,
    UsrNotFound,
    UsrNameAlreadyReg,
    UsrAlreadyReg,
    UsrRegWithAnotherName,
    ServNotFound,
    ServAlreadyReg,
    CannotCreateServInstance,
    TaskAlreadyReg,
    TaskNotFound,
    DbLoadFault,
    DbSaveFault,
    HelpTopicNotFound,
    ParseErr(UnitParseErr),
    DrvErr(DrvErr),
    ServErr(ServErr)
}

pub struct KernDrv {
    pub cli: Box<dyn CLI>,
    pub disp: Box<dyn Disp>,
    pub time: Box<dyn Time>,
    pub rnd: Box<dyn Rnd>,
    pub mem: Box<dyn Mem>,
}

struct KernDataPool {
    base: Vec<Rc<UnitBase>>,
    strings: Vec<Rc<String>>,
    paths: Vec<Rc<Path>>,
    addrs: Vec<Rc<Addr>>,
    ints: Vec<Rc<BigInt>>,
    decs: Vec<Rc<BigRational>>,
    lists: Vec<Rc<Vec<Unit>>>,
    maps: Vec<Rc<Vec<(Unit, Unit)>>>
}

pub struct Kern {
    pub drv: KernDrv,
    pub term: Rc<Mutex<base::Term>>,
    pub ram_store: RamStore,
    
    // vnix
    users: Vec<Usr>,
    services: Vec<Serv>,

    data_pool: KernDataPool,

    last_task_id: usize,
    curr_task_id: usize,
    tasks_queue: Vec<Task>,
    tasks_running: Vec<Task>,
    tasks_signals: Vec<(usize, TaskSig)>,
    task_result: Vec<(usize, Maybe<Msg, KernErr>)>
}

impl Display for Addr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Addr::Local => write!(f, "loc"),
            Addr::Remote(addr) => write!(f,
                "{:#04x}:{:#04x}:{:#04x}:{:#04x}:{:#04x}:{:#04x}:{:#04x}:{:#04x}",
                addr[0], addr[1], addr[2], addr[3],
                addr[4], addr[5], addr[6], addr[7]
            )
        }
    }
}

impl Write for Kern {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let term = self.term.clone();
        term.lock().print(s, self).ok();
        Ok(())
    }
}

impl KernDrv {
    pub fn new(cli: Box<dyn CLI>, disp: Box<dyn Disp>, time: Box<dyn Time>, rnd: Box<dyn Rnd>, mem: Box<dyn Mem>) -> Self {
        KernDrv {
            cli,
            disp,
            time,
            rnd,
            mem
        }
    }
}

impl KernDataPool {
    fn new() -> Self {
        KernDataPool {
            base: Vec::new(),
            strings: Vec::new(),
            paths: Vec::new(),
            addrs: Vec::new(),
            ints: Vec::new(),
            decs: Vec::new(),
            lists: Vec::new(),
            maps: Vec::new(),
        }
    }

    fn new_or_find_ub(&mut self, base: &UnitBase) -> Rc<UnitBase> {
        let found = self.base.iter().find(|b| b.as_ref().eq(base));

        if let Some(found) = found {
            return found.clone()
        } else {
            let rc = Rc::new(base.clone());
            self.base.push(rc.clone());
            return rc
        }
    }

    fn new_or_find_str(&mut self, s: &String) -> Rc<String> {
        let found = self.strings.iter().find(|_s| _s.as_str() == s.as_str());

        if let Some(found) = found {
            return found.clone()
        } else {
            let rc = Rc::new(s.clone());
            self.strings.push(rc.clone());
            return rc
        }
    }

    fn new_or_find_path(&mut self, path: &Vec<String>) -> Rc<Vec<String>> {
        let found = self.paths.iter().find(|p| p.as_ref() == path);

        if let Some(found) = found {
            return found.clone()
        } else {
            let rc = Rc::new(path.clone());
            self.paths.push(rc.clone());
            return rc
        }
    }

    fn new_or_find_addr(&mut self, addr: &Addr) -> Rc<Addr> {
        let found = self.addrs.iter().find(|a| a.as_ref().eq(addr));

        if let Some(found) = found {
            return found.clone()
        } else {
            let rc = Rc::new(addr.clone());
            self.addrs.push(rc.clone());
            return rc
        }
    }

    fn new_or_find_int(&mut self, val: &BigInt) -> Rc<BigInt> {
        let found = self.ints.iter().find(|v| v.as_ref().eq(val));

        if let Some(found) = found {
            return found.clone()
        } else {
            let rc = Rc::new(val.clone());
            self.ints.push(rc.clone());
            return rc
        }
    }

    fn new_or_find_dec(&mut self, val: &BigRational) -> Rc<BigRational> {
        let found = self.decs.iter().find(|v| v.as_ref().eq(val));

        if let Some(found) = found {
            return found.clone()
        } else {
            let rc = Rc::new(val.clone());
            self.decs.push(rc.clone());
            return rc
        }
    }

    fn new_or_find_list(&mut self, lst: &Vec<Unit>) -> Rc<Vec<Unit>> {
        let found = self.lists.iter().find(|l| l.as_ref() == lst);

        if let Some(found) = found {
            return found.clone()
        } else {
            let lst = lst.iter().map(|u| self.new_or_get(u.clone())).collect::<Vec<_>>();
            let rc = Rc::new(lst);
            self.lists.push(rc.clone());
            return rc
        }
    }

    fn new_or_find_map(&mut self, map: &Vec<(Unit, Unit)>) -> Rc<Vec<(Unit, Unit)>> {
        let found = self.maps.iter().find(|m| m.as_ref() == map);

        if let Some(found) = found {
            return found.clone();
        } else {
            let map = map.iter().map(|(u0, u1)| (self.new_or_get(u0.clone()), self.new_or_get(u1.clone()))).collect::<Vec<_>>();
            let rc = Rc::new(map);
            self.maps.push(rc.clone());
            return rc
        }
    }

    fn new_or_get(&mut self, u: Unit) -> Unit {
        let u_b = u.get_base();

        let base = match u_b.as_ref() {
            UnitBase::None | UnitBase::Bool(..) | UnitBase::Byte(..) => Rc::unwrap_or_clone(u_b),
            UnitBase::Str(s) => UnitBase::Str(self.new_or_find_str(&s)),
            UnitBase::Ref(path) => UnitBase::Ref(self.new_or_find_path(&path)),
            UnitBase::Stream(msg, serv, addr) => {
                let msg = self.new_or_get(msg.clone());
                let serv = self.new_or_find_str(&serv);
                let addr = self.new_or_find_addr(&addr);

                UnitBase::Stream(msg, serv, addr)
            },
            UnitBase::Int(v) => UnitBase::Int(Int(self.new_or_find_int(&v.0))),
            UnitBase::Dec(v) => UnitBase::Dec(Dec(self.new_or_find_dec(&v.0))),
            UnitBase::Pair(u0, u1) => UnitBase::Pair(self.new_or_get(u0.clone()), self.new_or_get(u1.clone())),
            UnitBase::List(lst) => UnitBase::List(self.new_or_find_list(&lst)),
            UnitBase::Map(map) => UnitBase::Map(self.new_or_find_map(&map))
        };
        Unit::share(self.new_or_find_ub(&base))
    }
}

impl Kern {
    pub fn new(drv: KernDrv, term: Rc<Mutex<base::Term>>) -> Self {
        let kern = Kern {
            drv,
            ram_store: RamStore::default(),
            term,
            users: Vec::new(),
            services: Vec::new(),
            data_pool: KernDataPool::new(),
            last_task_id: 0,
            curr_task_id: 0,
            tasks_queue: Vec::new(),
            tasks_running: Vec::new(),
            tasks_signals: Vec::new(),
            task_result: Vec::new()
        };

        kern
    }

    pub fn new_unit(&mut self, u: Unit) -> Unit {
        self.data_pool.new_or_get(u)
    }

    pub fn reg_usr(&mut self, usr: Usr) -> Result<(), KernErr> {
        if self.users.iter().find(|u| u.name == usr.name && u.pub_key != usr.pub_key).is_some() {
            return Err(KernErr::UsrNameAlreadyReg);
        }

        if self.users.iter().find(|u| u.name == usr.name && u.pub_key == usr.pub_key).is_some() {
            return Err(KernErr::UsrAlreadyReg);
        }

        if self.users.iter().find(|u| u.name != usr.name && u.pub_key == usr.pub_key).is_some() {
            return Err(KernErr::UsrRegWithAnotherName);
        }

        self.users.push(usr);
        Ok(())
    }

    fn get_usr(&self, ath: &str) -> Result<Usr, KernErr> {
        self.users.iter().find(|usr| usr.name == ath).ok_or(KernErr::UsrNotFound).cloned()
    }

    pub fn reg_serv(&mut self, serv: Serv) -> Result<(), KernErr> {
        if self.services.iter().find(|s| s.info.name == serv.info.name).is_some() {
            return Err(KernErr::ServAlreadyReg);
        }

        self.services.push(serv);
        Ok(())
    }

    pub fn reg_task(&mut self, usr: &str, name: &str, run: TaskRun) -> Result<usize, KernErr> {
        self.tasks_queue.push(Task::new(usr.into(), name.into(), self.last_task_id, self.curr_task_id, run));
        self.last_task_id += 1;
        Ok(self.last_task_id - 1)
    }

    pub fn task_sig(&mut self, id: usize, sig: TaskSig) -> Result<(), KernErr> {
        self.tasks_signals.push((id, sig));
        Ok(())
    }

    fn get_serv(&self, name: &str) -> Result<&Serv, KernErr> {
        self.services.iter().find(|s| s.info.name == name).ok_or(KernErr::ServNotFound)
    }

    pub fn get_tasks_running(&self) -> Vec<Task> {
        self.tasks_running.clone()
    }

    pub fn get_task_running(&self) -> Option<Task> {
        self.tasks_running.iter().find(|t| t.id == self.curr_task_id).map(|t| t.clone())
    }

    pub fn get_task_result(&mut self, id: usize) -> Option<Maybe<Msg, KernErr>> {
        self.task_result.drain_filter(|(i, _)| *i == id).next().map(|(_, msg)| msg)
    }

    pub fn msg(&self, ath: &str, u: Unit) -> Result<Msg, KernErr> {
        let usr = self.get_usr(ath)?;
        Msg::new(usr, u)
    }

    fn help_serv(&self, ath: &str) -> Result<Msg, KernErr> {
        let serv = self.services.iter().map(|serv| Unit::str(&serv.info.name)).collect::<Vec<_>>();
        let u = Unit::map(&[(
            Unit::str("msg"),
            Unit::list(&serv)
        )]);

        self.msg(ath, u)
    }

    pub fn send<'a>(mtx: &'a Mutex<Self>, serv: String, msg: Msg) -> Maybe<ServHlrAsync<'a>, KernErr> {
        // verify msg
        let usr = mtx.lock().get_usr(&msg.ath)?;
        usr.verify(msg.msg.clone(), &msg.sign, &msg.hash)?;

        // prepare msg
        let tmp = mtx.lock();
        let serv = tmp.get_serv(serv.as_str())?;

        let help_msg = Unit::map(&[
            (Unit::str("msg"), Unit::str(&serv.help))
        ]);
        let help_msg = tmp.msg(&msg.ath, help_msg)?;

        // check help
        let topic = if let Some(topic) = msg.msg.clone().as_map_find("help").map(|u| u.as_str()).flatten() {
            Some(topic)
        } else if let Some(topic) = msg.msg.clone().as_str() {
            Some(topic)
        } else {
            None
        };

        if let Some(topic) = topic {
            match topic.as_str() {
                "info" | "help" => return Ok(Some(thread!({
                    yield;
                    Ok(Some(help_msg))
                }))),
                "serv" => return Ok(Some(thread!({
                    let out = mtx.lock().help_serv(&msg.ath).map(|m| Some(m));
                    yield;
                    out
                }))),
                _ => ()
            }
        }

        // send
        let inst = (serv.hlr)(msg, serv.info.clone(), mtx);
        Ok(Some(inst))
    }

    pub fn run<'a>(self) -> Result<(), KernErr> {
        let kern_mtx = Mutex::new(self);

        loop {
            let mut runs = kern_mtx.lock().tasks_queue.clone().into_iter().map(|t| {
                let task = t.clone();
                let run = t.run(&kern_mtx);

                (task, (run, false))
            }).collect::<Vec<_>>();

            kern_mtx.lock().tasks_queue = Vec::new();

            // run tasks
            for (task, _) in runs.iter() {
                kern_mtx.lock().tasks_running.push(task.clone());
            }

            loop {
                for (task, (run, done)) in &mut runs {
                    // check signals
                    {
                        let mut grd = kern_mtx.lock();

                        if let Some(sig) = grd.tasks_signals.iter().find(|(id, _)| *id == task.id).map(|(_, sig)| sig.clone()) {
                            match sig {
                                TaskSig::Kill => {
                                    writeln!(grd, "INFO vnix:kern: killed task `{}#{}`", task.name, task.id).map_err(|_| KernErr::DrvErr(DrvErr::CLI(CLIErr::Write)))?;
                                    grd.tasks_running.drain_filter(|t| t.id == task.id).next();
                                    grd.tasks_signals.drain_filter(|(id, _)| *id == task.id).next();
                                    *done = true
                                }
                            }
                        }
                    }

                    if *done {
                        continue;
                    }

                    // run task
                    kern_mtx.lock().curr_task_id = task.id;

                    if let GeneratorState::Complete(res) = Pin::new(run).resume(()) {
                        match &res {
                            Ok(..) => (), // writeln!(kern_mtx.lock(), "DEBG vnix:kern: done task `{}#{}`", task.name, task.id).map_err(|_| KernErr::DrvErr(DrvErr::CLI(CLIErr::Write)))?,
                            Err(e) => {
                                writeln!(kern_mtx.lock(), "ERR vnix:{}#{}: {:?}", task.name, task.id, e).map_err(|_| KernErr::DrvErr(DrvErr::CLI(CLIErr::Write)))?;
                            }
                        };

                        kern_mtx.lock().task_result.push((task.id, res));
                        kern_mtx.lock().tasks_running.drain_filter(|t| t.id == task.id).next();
                        *done = true;
                    }
                }

                // run new tasks
                if !kern_mtx.lock().tasks_queue.is_empty() {
                    let mut new_runs = kern_mtx.lock().tasks_queue.clone().into_iter().map(|t| {
                        let task = t.clone();
                        let run = t.run(&kern_mtx);

                        (task, (run, false))
                    }).collect::<Vec<_>>();

                    kern_mtx.lock().tasks_queue = Vec::new();

                    for (task, _) in new_runs.iter() {
                        kern_mtx.lock().tasks_running.push(task.clone());
                        // writeln!(kern_mtx.lock(), "DEBG vnix:kern: run task `{}#{}`", task.name, task.id).map_err(|_| KernErr::DrvErr(DrvErr::CLI(CLIErr::Write)))?;
                    }

                    runs.append(&mut new_runs);
                }

                // done
                if runs.iter().all(|(_, (_, done))| *done) {
                    break;
                }
            }
        }
    }
}
