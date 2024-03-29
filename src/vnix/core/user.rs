use alloc::format;
use alloc::string::{String, ToString};
use core::fmt::{Display, Formatter};

use sha3::{Digest, Sha3_256};
use p256::ecdsa::{SigningKey, VerifyingKey};
use p256::ecdsa::signature::{Signature, Signer, Verifier};

use base64ct::{Base64, Encoding};

use crate::vnix::core::driver::DrvErr;

use super::kern::{KernErr, Kern};
use super::unit::{Unit, UnitAsBytes};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Usr {
    pub name: String,
    pub pub_key: String, // sec1: elliptic curve
    priv_key: Option<String>
}

impl Display for Usr {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        if self.name.contains(" ") {
            write!(f, "{{ath:`{}` pub:{} priv:{}}}", self.name, self.pub_key, if self.priv_key.is_some() {".."} else {"-"})
        } else {
            write!(f, "{{ath:{} pub:{} priv:{}}}", self.name, self.pub_key, if self.priv_key.is_some() {".."} else {"-"})
        }
    }
}

impl Usr {
    pub fn new(name: &str, kern: &mut Kern) -> Result<(Self, String), KernErr> {
        // gen private key
        let mut priv_key_b: [u8; 32] = [0; 32];
        kern.drv.rnd.get_bytes(&mut priv_key_b).map_err(|e| KernErr::DrvErr(DrvErr::Rnd(e)))?;

        let p = SigningKey::from_bytes(&priv_key_b).map_err(|_| KernErr::CreatePrivKeyFault)?;

        // gen public key
        let v = VerifyingKey::from(&p);
        let pub_key_b: [u8; 33] = v.to_encoded_point(true).as_bytes().try_into().map_err(|_| KernErr::CreatePubKeyFault)?;

        // encode base64
        let priv_key = Base64::encode_string(&priv_key_b); 
        let pub_key = Base64::encode_string(&pub_key_b);

        let out = format!("{{ath:`{}` pub:`{}` priv:`{}`}}", name, pub_key, priv_key);

        Ok((
            Usr {
                name: name.into(),
                priv_key: Some(priv_key),
                pub_key
            },
            out
        ))
    }

    pub fn guest(name: &str, pub_key: &str) -> Result<Self, KernErr> {
        Ok(Usr {
            name: name.to_string(),
            priv_key: None,
            pub_key: pub_key.to_string()
        })
    }

    pub fn login(name: &str, priv_key:&str, pub_key: &str) -> Result<Self, KernErr> {
        Ok(Usr {
            name: name.to_string(),
            priv_key: Some(priv_key.to_string()),
            pub_key: pub_key.to_string()
        })
    }

    pub fn sign(&self, u: Unit) -> Result<String, KernErr> {
        if let Some(priv_key_s) = &self.priv_key {
            let priv_key_b = Base64::decode_vec(priv_key_s.as_str()).map_err(|_| KernErr::DecodeFault)?;
            let priv_key = SigningKey::from_bytes(priv_key_b.as_slice()).map_err(|_| KernErr::CreatePrivKeyFault)?;

            let msg = u.clone().as_bytes();

            let sign_b = priv_key.sign(&msg);
            let sign = Base64::encode_string(&sign_b.as_bytes());

            return Ok(sign)
        }
        Err(KernErr::SignFault)
    }

    pub fn verify(&self, u: Unit, sign: &str, hash: &str) -> Result<(), KernErr> {
        let sign_b = Base64::decode_vec(sign).map_err(|_| KernErr::DecodeFault)?;
        let sign = Signature::from_bytes(&sign_b.as_slice()).map_err(|_| KernErr::SignVerifyFault)?;

        let pub_key_b = Base64::decode_vec(self.pub_key.as_str()).map_err(|_| KernErr::DecodeFault)?;
        let pub_key = VerifyingKey::from_sec1_bytes(&pub_key_b.as_slice()).map_err(|_| KernErr::CreatePubKeyFault)?;

        let msg = u.clone().as_bytes();

        let h = Sha3_256::digest(&msg);
        let _hash = Base64::encode_string(&h[..]);

        if _hash != hash {
            return Err(KernErr::HashVerifyFault);
        }

        pub_key.verify(&msg, &sign).map_err(|_| KernErr::SignVerifyFault)
    }
}
