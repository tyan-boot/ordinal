mod ffi;
use libc::{ioctl, open, O_RDONLY};
use ffi::*;
use anyhow::Result;
use std::ffi::CString;
use std::convert::TryInto;

pub struct Smart {
    fd: i32,
}

impl Smart {
    fn sg_io(
        &self,
        cdb: &mut [u8],
        in_data: Option<&[u8]>,
        out_data: Option<&mut [u8]>,
    ) -> Result<(SgIoHdr, [u8; 32])> {
        let mut hdr = SgIoHdr::default();
        let mut sense = [0u8; 32];

        hdr.cmd_len = cdb.len() as u8;

        hdr.mx_sb_len = sense.len() as u8;
        hdr.cmdp = cdb.as_mut_ptr();
        hdr.sbp = sense.as_mut_ptr();

        match (in_data, out_data) {
            (Some(in_data), None) => {
                hdr.dxfer_direction = SG_DXFER_TO_DEV;
                hdr.dxfer_len = in_data.len() as u32;
                hdr.dxferp = in_data.as_ptr() as *mut _; // safe, no write to in_data
            }
            (None, Some(out_data)) => {
                hdr.dxfer_direction = SG_DXFER_FROM_DEV;
                hdr.dxfer_len = out_data.len() as u32;
                hdr.dxferp = out_data.as_mut_ptr() as *mut _;
            }
            (None, None) => {
                hdr.dxfer_direction = SG_DXFER_NONE;
            }
            (Some(_), Some(_)) => {
                anyhow::bail!("only one direction allowed");
            }
        }

        let r = unsafe { ioctl(self.fd, SG_IO, &mut hdr) };
        anyhow::ensure!(r >= 0, "ioctl failed {}", std::io::Error::last_os_error());

        Ok((hdr, sense))
    }

    /// Open device with given path
    ///
    /// **Require root**
    pub fn open(device: impl AsRef<str>) -> Result<Smart> {
        let device = device.as_ref();

        let device_ffi = CString::new(device)?;

        let fd = unsafe { open(device_ffi.as_ptr(), O_RDONLY) };

        anyhow::ensure!(
            fd > 0,
            "open {} failed: {}",
            device,
            std::io::Error::last_os_error()
        );

        Ok(Smart { fd })
    }

    pub fn smart(&self) -> Result<()> {
        let mut cdb = ffi::build_ata_passthrough12(
            AtaCmd::SmartFunctionSet,
            Protocol::PioIn,
            SmartSubCmd::ReadAttr as u16,
            1,
            0,
            0b11000010_01001111
        );

        let mut buffer = Vec::with_capacity(512);
        buffer.resize(512, 0);
        println!("cdb = {:X?}", cdb);
        let (hdr, sense) = self.sg_io(&mut cdb, None, Some(&mut buffer))?;

        parse_smart_attributes(&buffer);


        Ok(())
    }
}

fn parse_smart_attributes(raw: &[u8]) {
    let version = u16::from_le_bytes(raw[0..2].try_into().unwrap());
    dbg!(version);
    let raw = &raw[2..];
    for idx in 0..30 {
        let attr = &raw[idx * 12..];

        let id = attr[0];
        let status = u16::from_le_bytes(attr[1..3].try_into().unwrap());
        let value = attr[3];
        let vendor = u64::from_le_bytes(attr[4..12].try_into().unwrap());

        dbg!(id, status, value, vendor);
    }

}

mod test {
    use crate::smart::Smart;

    #[test]
    fn test_smart() {
        let d = Smart::open("/dev/sda").unwrap();
        let a = 111f32;

        d.smart().unwrap();
    }
}