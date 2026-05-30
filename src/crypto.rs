use ed25519_dalek::{SigningKey, VerifyingKey, Signer};
use rand::rngs::OsRng;
use crate::eku::{Eku, EkuHeader};

pub struct HopeKeyPair { 
    pub signing: SigningKey, 
    pub verifying: VerifyingKey 
}

impl HopeKeyPair {
    pub fn generate() -> Self {
        let signing = SigningKey::generate(&mut OsRng);
        let verifying = signing.verifying_key();
        Self { signing, verifying }
    }

    pub fn verify(&self, eku: &Eku) -> bool {
        use ed25519_dalek::Verifier;
        use ed25519_dalek::Signature;
        let mut msg = Vec::new();
        let mut header_bytes = [0u8; std::mem::size_of::<EkuHeader>()];
        unsafe {
            std::ptr::copy_nonoverlapping(
                &eku.header as *const EkuHeader as *const u8,
                header_bytes.as_mut_ptr(),
                std::mem::size_of::<EkuHeader>(),
            );
        }
        msg.extend_from_slice(&header_bytes);
        msg.extend_from_slice(&eku.payload);
        let sig = Signature::from_bytes(&eku.signature);
        self.verifying.verify(&msg, &sig).is_ok()
    }

    pub fn sign(&self, eku: &mut Eku) {
        let mut msg = Vec::new();
        let mut header_bytes = [0u8; std::mem::size_of::<EkuHeader>()];
        unsafe {
            std::ptr::copy_nonoverlapping(
                &eku.header as *const EkuHeader as *const u8,
                header_bytes.as_mut_ptr(),
                std::mem::size_of::<EkuHeader>(),
            );
        }
        msg.extend_from_slice(&header_bytes);
        msg.extend_from_slice(&eku.payload);
        let sig = self.signing.sign(&msg);
        eku.signature.copy_from_slice(sig.to_bytes().as_ref());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eku::{Eku, EkuHeader, EkuType};

    fn make_eku() -> Eku {
        let sender_id: [u8; 16] = *b"TEST-SENDER-0001";
        let chain_ref: [u8; 16] = *b"TEST-CHAIN-00000";
        let header = EkuHeader::new(EkuType::Execute, 0x01, sender_id, 1, chain_ref, 4);
        Eku::new(header, b"test".to_vec())
    }

    #[test]
    fn keypair_generates() {
        let kp = HopeKeyPair::generate();
        assert_ne!(kp.verifying.to_bytes(), [0u8; 32]);
    }

    #[test]
    fn sign_and_verify() {
        let kp = HopeKeyPair::generate();
        let mut eku = make_eku();
        assert!(!eku.is_signed());
        kp.sign(&mut eku);
        assert!(eku.is_signed());
        assert!(kp.verify(&eku));
    }

    #[test]
    fn tampered_payload_fails_verify() {
        let kp = HopeKeyPair::generate();
        let mut eku = make_eku();
        kp.sign(&mut eku);
        eku.payload[0] ^= 0xFF; // megrontjuk
        assert!(!kp.verify(&eku));
    }
}
