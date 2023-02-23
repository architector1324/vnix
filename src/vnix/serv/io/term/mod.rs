mod text;

use core::pin::Pin;
use core::ops::{Generator, GeneratorState};

use spin::Mutex;
use alloc::sync::Arc;

use alloc::vec;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::string::String;

use crate::driver::CLIErr;
use crate::vnix::core::msg::Msg;
use crate::vnix::core::unit::{Unit, FromUnit, SchemaStr, Schema, SchemaMapEntry, SchemaUnit, SchemaOr, SchemaSeq, Or};
use crate::vnix::core::kern::{Kern, KernErr};
use crate::vnix::core::serv::{ServHlrAsync, Serv, ServHlr, ServHelpTopic};


#[derive(Debug, Clone)]
pub enum ActMode {
    Cli,
    Gfx,
}

#[derive(Debug)]
pub struct TermBase {
    pos: (usize, usize)
}

pub struct TermActAsync<'a>(Box<dyn Generator<Yield = (), Return = Result<Unit, KernErr>> + 'a>);

pub trait TermAct {
    fn act<'a>(self, orig: Arc<Msg>, msg: Unit, term: Arc<Term>, kern: &'a Mutex<Kern>) -> TermActAsync<'a>;
}

#[derive(Debug, Clone)]
enum ActKind {
    Cls,
    Say(text::Say)
}

#[derive(Debug, Clone)]
struct Act {
    kind: ActKind,
    mode: ActMode
}

#[derive(Debug)]
pub struct Term {
    acts: Option<Vec<Act>>
}

impl Term {
    fn clear(&self, mode: &ActMode, kern: &mut Kern) -> Result<(), CLIErr> {
        match mode {
            ActMode::Cli => kern.drv.cli.clear()?,
            ActMode::Gfx => todo!()
        }
        kern.term.pos = (0, 0);

        Ok(())
    }

    fn clear_line(&self, mode: &ActMode, kern: &mut Kern) -> Result<(), CLIErr> {
        match mode {
            ActMode::Cli => write!(kern.drv.cli, "\r").map_err(|_| CLIErr::Clear)?,
            ActMode::Gfx => todo!()
        }
        Ok(())
    }

    fn print(&self, out: &str, mode: &ActMode, kern: &mut Kern) ->  Result<(), CLIErr> {
        match mode {
            ActMode::Cli => {
                let (w, _) = kern.drv.cli.res()?;

                for ch in out.chars() {
                    if ch == '\n' {
                        kern.term.pos.1 += 1;
                        kern.term.pos.0 = 0;
                    } else if ch == '\r' {
                        self.clear_line(mode, kern)?;
                    } else if ch == '\u{8}' {
                        if kern.term.pos.0 == 0 && kern.term.pos.1 > 0 {
                            kern.term.pos.1 -= 1;
                        } else {
                            kern.term.pos.0 -= 1;
                        }
                    } else {
                        kern.term.pos.0 += 1;
                    }

                    if kern.term.pos.0 >= w {
                        kern.term.pos.1 += 1;
                        kern.term.pos.0 = 0;
                    }

                    write!(kern.drv.cli, "{}", ch).map_err(|_| CLIErr::Write)?;
                }
            },
            ActMode::Gfx => todo!()
        }
        Ok(())
    }
}

impl Default for TermBase {
    fn default() -> Self {
        TermBase {
            pos: (0, 0)
        }
    }
}

impl Default for Term {
    fn default() -> Self {
        Term {
            acts: None
        }
    }
}

impl FromUnit for Act {
    fn from_unit_loc(u: &Unit) -> Option<Self> {
        Self::from_unit(u, u)
    }

    fn from_unit(glob: &Unit, u: &Unit) -> Option<Self> {
        let schm = SchemaOr(
            SchemaStr,
            SchemaUnit
        );

        schm.find_deep(glob, u).and_then(|or| {
            match or {
                Or::First(s) =>
                    match s.as_str() {
                        "cls" => Some(Act {
                            kind: ActKind::Cls,
                            mode: ActMode::Cli
                        }),
                        "cls.gfx" => Some(Act {
                            kind: ActKind::Cls,
                            mode: ActMode::Gfx
                        }),
                        "say" => Some(Act {
                            kind: ActKind::Say(text::Say {
                                msg: Unit::Ref(vec!["msg".into()]),
                                shrt: None,
                                nl: false,
                                mode: text::SayMode::Norm,
                                act_mode: ActMode::Cli
                            }),
                            mode: ActMode::Cli
                        }),
                        "say.gfx" => Some(Act {
                            kind: ActKind::Say(text::Say {
                                msg: Unit::Ref(vec!["msg".into()]),
                                shrt: None,
                                nl: false,
                                mode: text::SayMode::Norm,
                                act_mode: ActMode::Gfx
                            }),
                            mode: ActMode::Cli
                        }),
                        "say.fmt" => Some(Act {
                            kind: ActKind::Say(text::Say {
                                msg: Unit::Ref(vec!["msg".into()]),
                                shrt: None,
                                nl: false,
                                mode: text::SayMode::Fmt,
                                act_mode: ActMode::Cli
                            }),
                            mode: ActMode::Cli
                        }),
                        "say.fmt.gfx" => Some(Act {
                            kind: ActKind::Say(text::Say {
                                msg: Unit::Ref(vec!["msg".into()]),
                                shrt: None,
                                nl: false,
                                mode: text::SayMode::Fmt,
                                act_mode: ActMode::Gfx
                            }),
                            mode: ActMode::Cli
                        }),
                        _ => None
                    },
                Or::Second(u) => {
                    if let Some(say) = text::Say::from_unit(glob, &u) {
                        return Some(Act {
                            mode: say.act_mode.clone(),
                            kind: ActKind::Say(say)
                        });
                    }

                    None
                }
            }
        })
    }
}

impl FromUnit for Term {
    fn from_unit_loc(u: &Unit) -> Option<Self> {
        let mut term = Term::default();

        let schm = SchemaOr(
            SchemaSeq(SchemaUnit),
            SchemaOr(
                SchemaMapEntry(
                    Unit::Str("term".into()),
                    SchemaOr(
                        SchemaSeq(SchemaUnit),
                        SchemaUnit
                    )
                ),
                SchemaUnit
            )
        );

        term.acts = schm.find_loc(u).and_then(|or| {
            let lst = match or {
                Or::First(seq) => seq,
                Or::Second(or) =>
                    match or {
                        Or::First(or) =>
                            match or {
                                Or::First(seq) => seq,
                                Or::Second(act) => vec![act]
                            }
                        Or::Second(act) => vec![act]
                    }
            };

            let acts = lst.into_iter().map(|act| Act::from_unit(&u, &act)).collect::<Option<Vec<_>>>()?;
            Some(acts)
        });

        Some(term)
    }
}

impl TermAct for Act {
    fn act<'a>(self, orig: Arc<Msg>, msg: Unit, term: Arc<Term>, kern: &'a Mutex<Kern>) -> TermActAsync<'a> {
        match self.kind {
            ActKind::Cls => TermActAsync(Box::new(move || {
                term.clear(&self.mode, &mut kern.lock()).map_err(|e| KernErr::CLIErr(e))?;
                yield;

                Ok(msg)
            })),
            ActKind::Say(say) => say.act(orig, msg, term, kern)
        }
    }
}

impl ServHlr for Term {
    fn help<'a>(self, ath: String, topic: ServHelpTopic, kern: &'a Mutex<Kern>) -> ServHlrAsync<'a> {
        let hlr = move || {
            let u = match topic {
                ServHelpTopic::Info => Unit::Str("Terminal I/O service\nExample: hello@io.term\nFor gfx mode: (say.gfx hello)@io.term".into())
            };

            let m = Unit::Map(vec![(
                Unit::Str("msg".into()),
                u
            )]);
    
            let out = kern.lock().msg(&ath, m).map(|msg| Some(msg));
            yield;

            out
        };
        ServHlrAsync(Box::new(hlr))
    }

    fn handle<'a>(self, msg: Msg, _serv: Serv, kern: &'a Mutex<Kern>) -> ServHlrAsync<'a> {
        let hlr = move || {
            writeln!(kern.lock().drv.cli, "io.term: {:?}", self.acts);

            if let Some(acts) = self.acts.clone() {
                let mut out_u = msg.msg.clone();

                let term = Arc::new(self);
                let msg = Arc::new(msg);

                for act in acts {
                    let mut gen = Box::into_pin(act.act(msg.clone(), out_u.clone(), term.clone(), kern).0);

                    loop {
                        if let GeneratorState::Complete(res) = Pin::new(&mut gen).resume(()) {
                            out_u = out_u.merge(res?);
                            break;
                        }
                        yield;
                    }
                    yield;
                }

                let msg = kern.lock().msg(&msg.ath, out_u)?;
                return Ok(Some(msg))
            }

            Ok(Some(msg))
        };
        ServHlrAsync(Box::new(hlr))
    }
}