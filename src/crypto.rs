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
