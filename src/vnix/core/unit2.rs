use alloc::format;
use alloc::rc::Rc;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::string::String;

use core::pin::Pin;
use core::fmt::Display;
use core::ops::{Generator, GeneratorState};

use num::bigint::BigInt;
use num::rational::BigRational;
use spin::Mutex;

use crate::driver::{MemSizeUnits, Mem};
use crate::{thread, thread_await};

use super::kern::{Addr, KernErr, Kern};
use super::task::ThreadAsync;


#[derive(Debug, PartialEq, Clone)]
pub enum Int {
    Small(i32),
    Nat(u32),
    Big(Rc<BigInt>)
}

#[derive(Debug, PartialEq, Clone)]
pub enum Dec {
    Small(f32),
    Big(Rc<BigRational>)
}

pub type Path = Vec<String>;

#[derive(Debug, PartialEq, Clone)]
pub enum UnitType {
    None,
    Bool(bool),
    Byte(u8),
    Int(Int),
    Dec(Dec),
    Str(Rc<String>),
    Ref(Rc<Path>),
    Stream(Unit, String, Addr),
    Pair(Unit, Unit),
    List(Rc<Vec<Unit>>),
    Map(Rc<Vec<(Unit, Unit)>>)
}

#[derive(Debug, PartialEq, Clone)]
pub struct Unit(Rc<UnitType>);

pub trait UnitNew {
    fn none() -> Unit;
    fn bool(v: bool) -> Unit;
    fn byte(v: u8) -> Unit;
    fn int(v: i32) -> Unit;
    fn uint(v: u32) -> Unit;
    fn int_big(v: BigInt) -> Unit;
    fn dec(v: f32) -> Unit;
    fn dec_big(v: BigRational) -> Unit;
    fn str(s: &str) -> Unit;
    fn path(path: &[&str]) -> Unit;
    fn stream_loc(u: Unit, serv: &str) -> Unit;
    fn stream(u: Unit, serv: &str, addr: Addr) -> Unit;
    fn pair(u0: Unit, u1: Unit) -> Unit;
    fn list(lst: &[Unit]) -> Unit;
    fn map(map: &[(Unit, Unit)]) -> Unit;
}

pub trait UnitAs {
    fn as_none(self) -> Option<()>;
    fn as_bool(self) -> Option<bool>;
    fn as_byte(self) -> Option<u8>;
    fn as_int(self) -> Option<i32>;
    fn as_uint(self) -> Option<u32>;
    fn as_int_big(self) -> Option<Rc<BigInt>>;
    fn as_dec(self) -> Option<f32>;
    fn as_dec_big(self) -> Option<Rc<BigRational>>;
    fn as_str(self) -> Option<Rc<String>>;
    fn as_path(self) -> Option<Rc<Path>>;
    fn as_stream(self) -> Option<(Unit, String, Addr)>;
    fn as_pair(self) -> Option<(Unit, Unit)>;
    fn as_list(self) -> Option<Rc<Vec<Unit>>>;
    fn as_map(self) -> Option<Rc<Vec<(Unit, Unit)>>>;
    fn as_map_find(self, sch: &str) -> Option<Unit>;
}

pub type UnitReadAsync<'a> = ThreadAsync<'a, Result<Option<(Unit, Rc<String>)>, KernErr>>;

pub trait UnitReadAsyncI {
    fn read_async<'a>(self, ath: Rc<String>, orig: Unit, kern: &'a Mutex<Kern>) -> UnitReadAsync<'a>;
    fn as_map_find_async<'a>(self, sch: String, ath: Rc<String>, orig: Unit, kern: &'a Mutex<Kern>) -> UnitReadAsync<'a>;
}

impl UnitNew for Unit {
    fn none() -> Unit {
        Unit::new(UnitType::None)
    }

    fn bool(v: bool) -> Unit {
        Unit::new(UnitType::Bool(v))
    }

    fn byte(v: u8) -> Unit {
        Unit::new(UnitType::Byte(v))
    }

    fn int(v: i32) -> Unit {
        Unit::new(UnitType::Int(Int::Small(v)))
    }

    fn uint(v: u32) -> Unit {
        Unit::new(UnitType::Int(Int::Nat(v)))
    }

    fn int_big(v: BigInt) -> Unit {
        Unit::new(UnitType::Int(Int::Big(Rc::new(v))))
    }

    fn dec(v: f32) -> Unit {
        Unit::new(UnitType::Dec(Dec::Small(v)))
    }

    fn dec_big(v: BigRational) -> Unit {
        Unit::new(UnitType::Dec(Dec::Big(Rc::new(v))))
    }

    fn str(s: &str) -> Unit {
        Unit::new(UnitType::Str(Rc::new(s.into())))
    }

    fn path(path: &[&str]) -> Unit {
        Unit::new(UnitType::Ref(Rc::new(path.into_iter().cloned().map(|s| format!("{s}")).collect())))
    }

    fn stream_loc(u: Unit, serv: &str) -> Unit {
        Unit::new(UnitType::Stream(u, serv.into(), Addr::Local))
    }

    fn stream(u: Unit, serv: &str, addr: Addr) -> Unit {
        Unit::new(UnitType::Stream(u, serv.into(), addr))
    }

    fn pair(u0: Unit, u1: Unit) -> Unit {
        Unit::new(UnitType::Pair(u0, u1))
    }

    fn list(lst: &[Unit]) -> Unit {
        Unit::new(UnitType::List(Rc::new(lst.to_vec())))
    }

    fn map(map: &[(Unit, Unit)]) -> Unit {
        Unit::new(UnitType::Map(Rc::new(map.to_vec())))
    }
}

impl UnitAs for Unit {
    fn as_none(self) -> Option<()> {
        if let UnitType::None = self.0.as_ref() {
            return Some(())
        }
        None
    }

    fn as_bool(self) -> Option<bool> {
        if let UnitType::Bool(v) = self.0.as_ref() {
            return Some(*v)
        }
        None
    }

    fn as_byte(self) -> Option<u8> {
        if let UnitType::Byte(v) = self.0.as_ref() {
            return Some(*v)
        }
        None
    }

    fn as_int(self) -> Option<i32> {
        if let UnitType::Int(v) = self.0.as_ref() {
            if let Int::Small(v) = v {
                return Some(*v)
            }
        }
        None
    }

    fn as_uint(self) -> Option<u32> {
        if let UnitType::Int(v) = self.0.as_ref() {
            if let Int::Nat(v) = v {
                return Some(*v)
            }
        }
        None
    }

    fn as_int_big(self) -> Option<Rc<BigInt>> {
        if let UnitType::Int(v) = self.0.as_ref() {
            if let Int::Big(v) = v {
                return Some(v.clone())
            }
        }
        None
    }

    fn as_dec(self) -> Option<f32> {
        if let UnitType::Dec(v) = self.0.as_ref() {
            if let Dec::Small(v) = v {
                return Some(*v)
            }
        }
        None
    }

    fn as_dec_big(self) -> Option<Rc<BigRational>> {
        if let UnitType::Dec(v) = self.0.as_ref() {
            if let Dec::Big(v) = v {
                return Some(v.clone())
            }
        }
        None
    }

    fn as_str(self) -> Option<Rc<String>> {
        if let UnitType::Str(s) = self.0.as_ref() {
            return Some(s.clone())
        }
        None
    }

    fn as_path(self) -> Option<Rc<Path>> {
        if let UnitType::Ref(path) = self.0.as_ref() {
            return Some(path.clone())
        }
        None
    }

    fn as_stream(self) -> Option<(Unit, String, Addr)> {
        if let UnitType::Stream(u, serv, addr) = self.0.as_ref() {
            return Some((u.clone(), serv.clone(), addr.clone()))
        }
        None
    }

    fn as_pair(self) -> Option<(Unit, Unit)> {
        if let UnitType::Pair(u0, u1) = self.0.as_ref() {
            return Some((u0.clone(), u1.clone()))
        }
        None
    }

    fn as_list(self) -> Option<Rc<Vec<Unit>>> {
        if let UnitType::List(lst) = self.0.as_ref() {
            return Some(lst.clone())
        }
        None
    }

    fn as_map(self) -> Option<Rc<Vec<(Unit, Unit)>>> {
        if let UnitType::Map(map) = self.0.as_ref() {
            return Some(map.clone())
        }
        None
    }

    fn as_map_find(self, sch: &str) -> Option<Unit> {
        if let UnitType::Map(map) = self.0.as_ref() {
            return map.iter()
                .filter_map(|(u0, u1)| Some((u0.clone().as_str()?, u1.clone())))
                .find_map(|(s, u)| {
                    if Rc::unwrap_or_clone(s) == sch {
                        return Some(u)
                    }
                    None
                })
        }
        None
    }
}

impl UnitReadAsyncI for Unit {
    fn read_async<'a>(self, ath: Rc<String>, orig: Unit, kern: &'a Mutex<Kern>) -> UnitReadAsync<'a> {
        thread!({
            match self.0.as_ref() {
                UnitType::Ref(path) => {
                    yield;
                    todo!()
                },
                UnitType::Stream(msg, serv, _addr) => {
                    todo!()
                },
                _ => Ok(Some((self.clone(), ath)))
            }
        })
    }

    fn as_map_find_async<'a>(self, sch: String, ath: Rc<String>, orig: Unit, kern: &'a Mutex<Kern>) -> UnitReadAsync<'a> {
        thread!({
            if let Some(msg) = self.as_map_find(&sch) {
                return thread_await!(msg.read_async(ath, orig, kern))
            }
            Ok(None)
        })
    }
}

impl Display for Unit {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.0.as_ref() {
            UnitType::None => write!(f, "-"),
            UnitType::Bool(v) => write!(f, "{}", if *v {"t"} else {"f"}),
            UnitType::Byte(v) => write!(f, "{:#02x}", *v),
            UnitType::Int(v) =>
                match v {
                    Int::Small(v) => write!(f, "{v}"),
                    Int::Nat(v) => write!(f, "{v}"),
                    Int::Big(v) => write!(f, "{v}")
                }
            _ => todo!()
        }
    }
}

impl Unit {
    fn new(t: UnitType) -> Unit {
        Unit(Rc::new(t))
    }

    pub fn ptr(&self) -> *const UnitType {
        unsafe {
            Rc::as_ptr(&self.0)
        }
    }

    pub fn size(&self, units: MemSizeUnits) -> usize {
        let size = core::mem::size_of::<UnitType>() + match self.0.as_ref() {
            UnitType::None | UnitType::Bool(..) | UnitType::Byte(..) => 0,
            UnitType::Int(v) =>
                match v {
                    Int::Small(..) | Int::Nat(..) => 0,
                    Int::Big(v) => v.to_bytes_le().1.len(),
                },
            UnitType::Dec(v) =>
                match v {
                    Dec::Small(..) => 0,
                    Dec::Big(v) => v.numer().to_bytes_le().1.len() + v.denom().to_bytes_le().1.len()
                },
            UnitType::Str(s) => s.len(),
            UnitType::Ref(path) => path.iter().fold(0, |prev, s| prev + s.len()),
            UnitType::Stream(msg, serv, addr) => msg.size(MemSizeUnits::Bytes) + serv.len(),
            UnitType::Pair(u0, u1) => u0.size(MemSizeUnits::Bytes) + u1.size(MemSizeUnits::Bytes),
            UnitType::List(lst) => lst.iter().fold(0, |prev, u| prev + u.size(MemSizeUnits::Bytes)),
            UnitType::Map(map) => map.iter().fold(0, |prev, (u0, u1)| prev + u0.size(MemSizeUnits::Bytes) + u1.size(MemSizeUnits::Bytes))
        };

        match units {
            MemSizeUnits::Bytes => size,
            MemSizeUnits::Kilo => size / 1024,
            MemSizeUnits::Mega => size / (1024 * 1024),
            MemSizeUnits::Giga => size / (1024 * 1024 * 1024)
        }
    }
}
