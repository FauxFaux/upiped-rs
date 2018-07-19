// safe mtu: 508

// bit 0: 0,  version
// bit 1-31:  key id
// 12 bytes:  nonce
// 476 bytes: body
// 16 bytes:  tag
// 508 - 12 - 16 - 4 = 476

use byteorder::ByteOrder;
use cast::u8;
use chacha20_poly1305_aead as cha;
use failure::Error;
use rand;
use rand::Rng;
use rand::RngCore;

const MTU: usize = 508;
type KeyId = u32;

// 6-bit opcode, 1 bit critical flag, 1-bit length extension
// 8-bit length

// space for the rand opcode
const PLAINTEXT_LEN: usize = 476;
const MAX_USER_DATA: usize = PLAINTEXT_LEN - 2;

fn pack(key_id: KeyId, data: &[u8]) -> Result<[u8; MTU], Error> {
    ensure!(data.len() <= MAX_USER_DATA, "too much user data");

    let mut packet = [0u8; MTU];
    ::byteorder::BigEndian::write_u32(&mut packet, key_id);
    packet[0] &= 0b0111_1111;

    let mut rng = rand::thread_rng();

    let nonce: [u8; 12] = rng.gen();
    packet[4..(12 + 4)].copy_from_slice(&nonce);

    let mut plaintext = [0u8; PLAINTEXT_LEN];
    let padding_required = PLAINTEXT_LEN - data.len();
    assert_ge!(padding_required, 2);
    write_opcode(&mut plaintext[..2], OpCode::Padding, padding_required)?;
    rng.fill_bytes(&mut plaintext[2..padding_required - 2]);
    plaintext[padding_required..].copy_from_slice(data);

    let tag = cha::encrypt(
        &[0u8; 32],
        &nonce,
        &[],
        &plaintext,
        &mut &mut packet[12 + 4..],
    )?;
    packet[12 + 4 + PLAINTEXT_LEN..].copy_from_slice(&tag);

    Ok(packet)
}

#[derive(Copy, Clone, Debug)]
enum OpCode {
    Padding = 64,
}

fn write_opcode(into: &mut [u8], code: OpCode, frame_len: usize) -> Result<(), Error> {
    ensure!(frame_len <= PLAINTEXT_LEN, "frame bigger than packet");
    into[0] = ((code as u8) << 1) | u8(frame_len >> 8)?;
    into[1] = u8(frame_len & 0xff)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::pack;
    use super::KeyId;

    #[test]
    fn smoke() {
        println!("{:?}", pack(0, &[]).unwrap().to_vec());
    }
}
