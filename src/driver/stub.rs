use alloc::vec::Vec;
use rand::{rngs::StdRng, SeedableRng, RngCore};

use crate::vnix::utils::Maybe;
use crate::vnix::core::driver::{DispErr, Disp, Rnd, RndErr, Mouse};

pub struct StubDisp;

impl Disp for StubDisp {
    fn res(&self) -> Result<(usize, usize), DispErr> {
        Ok((0, 0))
    }

    fn res_list(&self) -> Result<alloc::vec::Vec<(usize, usize)>, DispErr> {
        Ok(Vec::new())
    }

    fn set_res(&mut self, _res: (usize, usize)) -> Result<(), DispErr> {
        Ok(())
    }

    fn px(&mut self, _px: u32, _x: usize, _y: usize) -> Result<(), DispErr> {
        Ok(())
    }

    fn blk(&mut self, _pos: (i32, i32), _img_size: (usize, usize), _src: u32, _img: &[u32]) -> Result<(), DispErr> {
        Ok(())
    }

    fn fill(&mut self, _f: &dyn Fn(usize, usize) -> u32) -> Result<(), DispErr> {
        Ok(())
    }

    fn flush(&mut self) -> Result<(), DispErr> {
        Ok(())
    }

    fn flush_blk(&mut self, _pos: (i32, i32), _size: (usize, usize)) -> Result<(), DispErr> {
        Ok(())
    }

    fn mouse(&mut self, _block: bool) -> Maybe<Mouse, DispErr> {
        Ok(None)
    }
}

pub struct PRng(pub [u8; 32]);

impl Rnd for PRng {
    fn get_bytes(&mut self, buf: &mut [u8]) -> Result<(), RndErr> {
        let mut rng = StdRng::from_seed(self.0);

        rng.fill_bytes(buf);
        self.0 = buf[0..32].try_into().map_err(|_| RndErr::GetBytes)?;

        Ok(())
    }
}
