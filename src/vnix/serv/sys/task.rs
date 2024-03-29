use core::pin::Pin;
use core::slice::Iter;
use core::ops::{Coroutine, CoroutineState};

use spin::Mutex;

use alloc::rc::Rc;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::string::String;

use crate::vnix::utils::Maybe;
use crate::{thread, thread_await, read_async, as_map_find_async, maybe, as_map_find_as_async, as_async, maybe_ok, task_result};

use crate::vnix::core::msg::Msg;
use crate::vnix::core::kern::{Kern, KernErr};
use crate::vnix::core::task::{ThreadAsync, TaskRun, TaskSig, Task};
use crate::vnix::core::serv::{ServHlrAsync, ServInfo};
use crate::vnix::core::unit::{Unit, UnitReadAsyncI, UnitParse, UnitModify, UnitAs, UnitNew, UnitReadAsync, UnitTypeReadAsync};


pub const SERV_PATH: &'static str = "sys.task";
const SERV_HELP: &'static str = "{
    name:sys.task
    info:`Service for task management`
    tut:[
        {
            info:`Run task from stream`
            com:{sum:[1 2 3]}@math.calc@sys.task
            res:6
        }
        {
            info:`Run infinite loop task from stream`
            com:(task.loop (say a)@io.term)@sys.task
        }
        {
            info:`Run loop task from stream`
            com:(task.loop (5 (say a)@io.term))@sys.task
        }
        {
            info:`Run parallel task`
            com:(task.sep a@io.term)@sys.task
        }
        {
            info:`Run task chain with current message`
            com:{sum:[1 2 3] task:[math.calc io.term]}@sys.task
        }
        {
            info:`Run several parallel tasks`
            com:(task.sim [a@io.term b@io.term])@sys.task
        }
        {
            info:`Run sequence of tasks`
            com:(task.que [a@io.term b@io.term])@sys.task
        }
        {
            info:`Create sequence of tasks with messages sended to service`
            com:(task.stk [a b]@io.term)@sys.task
            alt:(task.que [a@io.term b@io.term])@sys.task
        }
        {
            info:`Get information about running tasks`
            com:get@sys.task
            res:{
                run:{
                    id:37
                    name:unit.read
                    usr:super
                    par.id:36
                }
                all:[
                    {
                        id:0
                        name:init.load
                        usr:super
                        par.id:0
                    }
                    {
                        id:9
                        name:unit.read
                        usr:super
                        par.id:0
                    }
                    {
                        id:15
                        name:unit.read
                        usr:super
                        par.id:9
                    }
                    {
                        id:32
                        name:unit.read
                        usr:super
                        par.id:15
                    }
                    {
                        id:33
                        name:unit.read
                        usr:super
                        par.id:32
                    }
                    {
                        id:36
                        name:unit.read
                        usr:super
                        par.id:33
                    }
                    {
                        id:37
                        name:unit.read
                        usr:super
                        par.id:36
                    }
                ]
                tree:{
                    id:0
                    name:init.load
                    usr:super
                    child:[
                        {
                            id:9
                            name:unit.read
                            usr:super
                            child:[
                                {
                                    id:15
                                    name:unit.read
                                    usr:super
                                    child:[
                                        {
                                            id:32
                                            name:unit.read
                                            usr:super
                                            child:[
                                                {
                                                    id:33
                                                    name:unit.read
                                                    usr:super
                                                    child:[
                                                        {
                                                            id:36
                                                            name:unit.read
                                                            usr:super
                                                            child:[
                                                                {
                                                                    id:37
                                                                    name:unit.read
                                                                    usr:super
                                                                    child:-
                                                                }
                                                            ]
                                                        }
                                                    ]
                                                }
                                            ]
                                        }
                                    ]
                                }
                            ]
                        }
                    ]
                }
            }
        }
        {
            info:`Get information about current running task`
            com:get.run@sys.task
            res:{
                id:71
                name:unit.read
                usr:super
                par.id:70
            }
        }
        {
            info:`Get list of running tasks`
            com:get.all@sys.task
            res:[
                {
                    id:0
                    name:init.load
                    usr:super
                    par.id:0
                }
                {
                    id:9
                    name:unit.read
                    usr:super
                    par.id:0
                }
                {
                    id:15
                    name:unit.read
                    usr:super
                    par.id:9
                }
                {
                    id:100
                    name:unit.read
                    usr:super
                    par.id:15
                }
                {
                    id:101
                    name:unit.read
                    usr:super
                    par.id:100
                }
                {
                    id:104
                    name:unit.read
                    usr:super
                    par.id:101
                }
                {
                    id:105
                    name:unit.read
                    usr:super
                    par.id:104
                }
            ]
        }
        {
            info:`Get tree of running tasks`
            com:get.tree@sys.task
            res:{
                id:0
                name:init.load
                usr:super
                child:[
                    {
                        id:9
                        name:unit.read
                        usr:super
                        child:[
                            {
                                id:15
                                name:unit.read
                                usr:super
                                child:[
                                    {
                                        id:134
                                        name:unit.read
                                        usr:super
                                        child:[
                                            {
                                                id:135
                                                name:unit.read
                                                usr:super
                                                child:[
                                                    {
                                                        id:138
                                                        name:unit.read
                                                        usr:super
                                                        child:[
                                                            {
                                                                id:139
                                                                name:unit.read
                                                                usr:super
                                                                child:-
                                                            }
                                                        ]
                                                    }
                                                ]
                                            }
                                        ]
                                    }
                                ]
                            }
                        ]
                    }
                ]
            }
        }
        {
            info:`Kill task by id`
            com:(kill 2)@sys.task
        }
    ]
    man:{
        task:{
            loop:{
                info:`Run loop task from stream`
                schm:[
                    (task.loop stream)
                    (task.loop (uint stream))
                    {task.loop:stream}
                    {task.loop:(uint stream)}
                ]
                tut:[@tut.1 @tut.2]
            }
            sep:{
                info:`Run parallel task`
                schm:[
                    (task.sep stream)
                    {task.sep:stream}
                ]
                tut:@tut.3
            }
            chain:{
                info:`Run task chain with current message`
                schm:{task:[serv]}
                tut:@tut.4
            }
            sim:{
                info:`Run several parallel tasks`
                schm:[
                    (task.sim [unit@serv])
                    {task.sim:[unit@serv]}
                ]
                tut:@tut.5
            }
            que:{
                info:`Run sequence of tasks`
                schm:[
                    (task.que [unit@serv])
                    {task.que:[unit@serv]}
                ]
                tut:@tut.6
            }
            stk:{
                info:`Create sequence of tasks with messages sended to service`
                schm:[
                    (task.stk [unit]@serv)
                    {task.stk:[unit]@serv}
                ]
                tut:@tut.7
            }
        }
        get:{
            info:`Get information about running tasks`
            schm:[
                get
                get.run
                get.all
                get.tree
            ]
            tut:[
                @tut.8
                @tut.9
                @tut.10
                @tut.11
            ]
        }
        kill:{
            info:`Kill task by id`
            schm:(kill uint)
            tut:@tut.12
        }
    }
}";

fn stream(ath: Rc<String>, orig: Unit, msg: Unit, kern: &Mutex<Kern>) -> UnitReadAsync {
    thread!({
        maybe_ok!(msg.clone().as_stream());

        let (msg, ath) = maybe!(read_async!(msg, ath, orig, kern));
        Ok(Some((msg, ath)))
    })
}

fn _loop(mut ath: Rc<String>, orig: Unit, msg: Unit, kern: &Mutex<Kern>) -> ThreadAsync<Maybe<Rc<String>, KernErr>> {
    thread!({
        let msg = if let Some(msg) = msg.clone().as_map_find("task.loop") {
            msg
        } else if let Some((s, msg)) = msg.clone().as_pair() {
            let (s, _ath) = maybe!(as_async!(s, as_str, ath, orig, kern));
            ath = _ath;

            if s.as_str() != "task.loop" {
                return Ok(None)
            }
            msg
        } else {
            return Ok(None)
        };

        // loop count
        if let Some((cnt, msg)) = msg.clone().as_pair() {
            let (cnt, mut ath) = maybe!(as_async!(cnt, as_uint, ath, orig, kern));

            for _ in 0..cnt {
                if let Some((_, _ath)) = read_async!(msg, ath, orig, kern)? {
                    ath = _ath;
                }
            }
            return Ok(Some(ath))
        }

        // infinite
        loop {
            read_async!(msg, ath, orig, kern)?;
        }
    })
}

fn separate(mut ath: Rc<String>, orig: Unit, msg: Unit, kern: &Mutex<Kern>) -> ThreadAsync<Maybe<Rc<String>, KernErr>> {
    thread!({
        let msg = if let Some(msg) = msg.clone().as_map_find("task.sep") {
            msg
        } else if let Some((s, msg)) = msg.clone().as_pair() {
            let (s, _ath) = maybe!(as_async!(s, as_str, ath, orig, kern));
            ath = _ath;

            if s.as_str() != "task.sep" {
                return Ok(None)
            }
            msg
        } else {
            return Ok(None)
        };

        // infinite
        if let Some((_msg, serv, _)) = msg.as_stream() {
            let run = TaskRun(_msg, serv);
            kern.lock().reg_task(&ath, "sys.task", run)?;
        }

        Ok(Some(ath))
    })
}

fn chain(ath: Rc<String>, orig: Unit, msg: Unit, kern: &Mutex<Kern>) -> UnitReadAsync {
    thread!({
        let (lst, mut ath) = maybe!(as_map_find_as_async!(msg, "task", as_list, ath, orig, kern));
        let mut _msg = if let Some((_msg, _ath)) = as_map_find_async!(msg, "msg", ath, orig, kern)? {
            ath = _ath;
            _msg
        } else {
            msg.clone()
        };

        for p in Rc::unwrap_or_clone(lst) {
            let (serv, _ath) = maybe!(as_async!(p, as_str, ath, orig, kern));
            let prev = _msg.clone();

            let run = TaskRun(_msg, Rc::unwrap_or_clone(serv));
            let id = kern.lock().reg_task(&_ath, "sys.task", run)?;

            let u = maybe_ok!(task_result!(id, kern)?);

            _msg = prev.merge_with(u.msg);
            ath = Rc::new(u.ath);
        }
        return Ok(Some((_msg, ath)))
    })
}

fn queue(ath: Rc<String>, orig: Unit, msg: Unit, kern: &Mutex<Kern>) -> ThreadAsync<Maybe<Rc<String>, KernErr>> {
    thread!({
        let (lst, mut ath) = if let Some((lst, ath)) =  as_map_find_as_async!(msg, "task.que", as_list, ath, orig, kern)? {
            (lst, ath)
        } else if let Some((s, lst)) = msg.as_pair() {
            let (s, ath) = maybe!(as_async!(s, as_str, ath, orig, kern));

            if s.as_str() != "task.que" {
                return Ok(None)
            }

            let (lst, ath) = maybe!(as_async!(lst, as_list, ath, orig, kern));
            (lst, ath)
        } else {
            return Ok(None)
        };

        for p in Rc::unwrap_or_clone(lst) {
            if let Some((_, _ath)) = read_async!(p, ath, orig, kern)? {
                ath = _ath;
            }
        }
        Ok(Some(ath))
    })
}

fn sim(ath: Rc<String>, orig: Unit, msg: Unit, kern: &Mutex<Kern>) -> ThreadAsync<Maybe<(), KernErr>> {
    thread!({
        let lst = if let Some((lst, _)) =  as_map_find_as_async!(msg, "task.sim", as_list, ath, orig, kern)? {
            lst
        } else if let Some((s, lst)) = msg.as_pair() {
            let (s, ath) = maybe!(as_async!(s, as_str, ath, orig, kern));

            if s.as_str() != "task.sim" {
                return Ok(None)
            }

            let (lst, _) = maybe!(as_async!(lst, as_list, ath, orig, kern));
            lst
        } else {
            return Ok(None)
        };

        for p in lst.iter() {
            if let Some((_msg, serv, _)) = p.clone().as_stream() {
                let run = TaskRun(_msg, serv);
                kern.lock().reg_task(&ath, "sys.task", run)?;
            }
        }
        Ok(None)
    })
}

fn stack(ath: Rc<String>, orig: Unit, msg: Unit, kern: &Mutex<Kern>) -> ThreadAsync<Maybe<Rc<String>, KernErr>> {
    thread!({
        // let (u, serv, _) = maybe_ok!(msg.as_map_find("task.stk").and_then(|u| u.as_stream()));
        let (u, serv, _) = if let Some((u, serv, addr)) = msg.clone().as_map_find("task.stk").and_then(|u| u.as_stream()) {
            (u, serv, addr)
        } else if let Some((s, msg)) = msg.clone().as_pair() {
            let (s, _) = maybe!(as_async!(s, as_str, ath, orig, kern));

            if s.as_str() != "task.stk" {
                return Ok(None)
            }
            maybe_ok!(msg.as_stream())
        } else {
            return Ok(None)
        };

        let (lst, mut ath) = maybe!(as_async!(u, as_list, ath, orig, kern));

        for p in Rc::unwrap_or_clone(lst) {
            let (msg, _ath) = maybe!(read_async!(p, ath, orig, kern));
            ath = _ath;

            let run = TaskRun(msg, serv.clone());
            let id = kern.lock().reg_task(&ath, "sys.task", run)?;

            if let Some(msg) = task_result!(id, kern)? {
                ath = Rc::new(msg.ath);
            }
        }
        Ok(Some(ath))
    })
}

fn run(ath: Rc<String>, orig: Unit, msg: Unit, kern: &Mutex<Kern>) -> UnitTypeReadAsync<Option<Unit>> {
    thread!({
        // loop
        if let Some(_ath) = thread_await!(_loop(ath.clone(), msg.clone(), orig.clone(), kern))? {
            if _ath != ath {
                return Ok(Some((Some(msg), ath)))
            }
            return Ok(Some((None, ath)))
        }

        // separate
        if let Some(_ath) = thread_await!(separate(ath.clone(), msg.clone(), orig.clone(), kern))? {
            if _ath != ath {
                return Ok(Some((Some(msg), ath)))
            }
            return Ok(Some((None, ath)))
        }

        // chain
        if let Some((msg, ath)) = thread_await!(chain(ath.clone(), msg.clone(), orig.clone(), kern))? {
            let msg = Unit::map(&[
                (Unit::str("msg"), msg)]
            );
            return Ok(Some((Some(msg), ath)))
        }
    
        // sim
        thread_await!(sim(ath.clone(), msg.clone(), orig.clone(), kern))?;
    
        // queue
        if let Some(_ath) = thread_await!(queue(ath.clone(), msg.clone(), orig.clone(), kern))? {
            if _ath != ath {
                return Ok(Some((Some(msg), ath)))
            }
            return Ok(Some((None, ath)))
        }

        // stack
        if let Some(_ath) = thread_await!(stack(ath.clone(), msg.clone(), orig.clone(), kern))? {
            if _ath != ath {
                return Ok(Some((Some(msg), ath)))
            }
            return Ok(Some((None, ath)))
        }

        // stream
        if let Some((msg, ath)) = thread_await!(stream(ath.clone(), msg.clone(), orig.clone(), kern))? {
            let msg = Unit::map(&[
                (Unit::str("msg"), msg)]
            );
            return Ok(Some((Some(msg), ath)))
        }

        Ok(None)
    })
}

fn get(ath: Rc<String>, _orig: Unit, msg: Unit, kern: &Mutex<Kern>) -> UnitReadAsync {
    thread!({
        let s = maybe_ok!(msg.as_str());

        let info = {
            let task = maybe_ok!(kern.lock().get_task_running());
            let tasks = kern.lock().get_tasks_running();

            let task_lst = tasks.iter().map(|t| {
                Unit::map(&[
                    (Unit::str("id"), Unit::uint(t.id as u32)),
                    (Unit::str("name"), Unit::str(&t.name)),
                    (Unit::str("usr"), Unit::str(&t.usr)),
                    (Unit::str("par.id"), Unit::uint(t.parent_id as u32))
                ])
            }).collect();

            let task_tree = {
                fn get_childs(root: &Task, tasks: Iter<Task>) -> Unit {
                    let childs = tasks.clone().filter_map(|t| {
                        if t.id != root.id && t.id != root.parent_id && t.parent_id == root.id {
                            return Some(Unit::map(&[
                                (Unit::str("id"), Unit::uint(t.id as u32)),
                                (Unit::str("name"), Unit::str(&t.name)),
                                (Unit::str("usr"), Unit::str(&t.usr)),
                                (Unit::str("child"), get_childs(t, tasks.clone()))
                            ]))
                        }
                        None
                    }).collect::<Vec<_>>();

                    if childs.len() == 0 {
                        Unit::none()
                    } else {
                        Unit::list_share(Rc::new(childs))
                    }
                }

                let root = maybe_ok!(tasks.iter().min_by(|a, b| a.id.cmp(&b.id)));
                Unit::map(&[
                    (Unit::str("id"), Unit::uint(root.id as u32)),
                    (Unit::str("name"), Unit::str(&root.name)),
                    (Unit::str("usr"), Unit::str(&root.usr)),
                    (Unit::str("child"), get_childs(root, tasks.iter()))
                ])
            };

            Unit::map(&[
                (
                    Unit::str("run"),
                    Unit::map(&[
                        (Unit::str("id"), Unit::uint(task.id as u32)),
                        (Unit::str("name"), Unit::str(&task.name)),
                        (Unit::str("usr"), Unit::str(&task.usr)),
                        (Unit::str("par.id"), Unit::uint(task.parent_id as u32))
                    ])
                ),
                (Unit::str("all"), Unit::list_share(Rc::new(task_lst))),
                (Unit::str("tree"), task_tree)
            ])
        };
        yield;

        // get
        let res = match s.as_str() {
            "get" => info,
            "get.run" => maybe_ok!(info.find(["run"].into_iter())),
            "get.all" => maybe_ok!(info.find(["all"].into_iter())),
            "get.tree" => maybe_ok!(info.find(["tree"].into_iter())),
            _ => return Ok(None)
        };
        Ok(Some((res, ath)))
    })
}

fn signal(ath: Rc<String>, orig: Unit, msg: Unit, kern: &Mutex<Kern>) -> ThreadAsync<Maybe<Rc<String>, KernErr>> {
    thread!({
        let (sig, id) = maybe_ok!(msg.as_pair());

        let (sig, ath) = maybe!(as_async!(sig, as_str, ath, orig, kern));
        let (id, ath) = maybe!(as_async!(id, as_uint, ath, orig, kern));

        match sig.as_str() {
            "kill" => kern.lock().task_sig(id as usize, TaskSig::Kill)?,
            _ => return Ok(None)
        }

        Ok(Some(ath))
    })
}

pub fn help_hlr(msg: Msg, _serv: ServInfo, kern: &Mutex<Kern>) -> ServHlrAsync {
    thread!({
        let s = maybe_ok!(msg.msg.clone().as_str());
        let help = Unit::parse(SERV_HELP.chars()).map_err(|e| KernErr::ParseErr(e))?.0;
        yield;

        let res = match s.as_str() {
            "help" => help,
            "help.name" => maybe_ok!(help.find(["name"].into_iter())),
            "help.info" => maybe_ok!(help.find(["info"].into_iter())),
            "help.tut" => maybe_ok!(help.find(["tut"].into_iter())),
            "help.man" => maybe_ok!(help.find(["man"].into_iter())),
            _ => return Ok(None)
        };

        let _msg = Unit::map(&[
            (Unit::str("msg"), res)
        ]);
        kern.lock().msg(&msg.ath, _msg).map(|msg| Some(msg))
    })
}

pub fn task_hlr(mut msg: Msg, _serv: ServInfo, kern: &Mutex<Kern>) -> ServHlrAsync {
    thread!({
        let ath = Rc::new(msg.ath.clone());
        let (_msg, mut ath) = maybe!(read_async!(msg.msg.clone(), ath, msg.msg.clone(), kern));

        // task
        if let Some((u, ath)) = thread_await!(run(ath.clone(), _msg.clone(), _msg.clone(), kern))? {
            let msg = _msg.clone().merge_with(maybe_ok!(u));
            return kern.lock().msg(&ath, msg).map(|msg| Some(msg))
        }

        // get
        if let Some((u, ath)) = thread_await!(get(ath.clone(), _msg.clone(), _msg.clone(), kern))? {
            let msg = Unit::map(&[
                (Unit::str("msg"), u)
            ]);
            return kern.lock().msg(&ath, msg).map(|msg| Some(msg))
        }

        // signal
        if let Some(_ath) = thread_await!(signal(ath.clone(), _msg.clone(), _msg.clone(), kern))? {
            if _ath != ath {
                ath = _ath;
                msg = kern.lock().msg(&ath, _msg.clone())?;
            }
            return Ok(Some(msg))
        }

        Ok(Some(msg))
    })
}
