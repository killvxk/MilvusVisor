//!
//! Advanced Configuration and Power Interface
//!
//! Supported ACPI Version 6.4

pub mod madt;

const RSDP_SIGNATURE: [u8; 8] = *b"RSD PTR ";
const XSDT_SIGNATURE: [u8; 4] = *b"XSDT";

const XSDT_STRUCT_SIZE: usize = core::mem::size_of::<XSDT>();

#[repr(C, packed)]
struct RSDP {
    signature: [u8; 8],
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    rsdt_address: u32,
    length: u32,
    xsdt_address: u64,
    ex_checksum: u32,
    reserved: [u8; 3],
}

#[repr(C, packed)]
struct XSDT {
    signature: [u8; 4],
    length: u32,
    revison: u8,
    checksum: u8,
    oem_id: [u8; 6],
    oem_table_id: u64,
    oem_revision: u32,
    creator_id: u32,
    creator_revision: u32,
    /* entries */
}

#[derive(Debug, Clone)]
pub struct GeneralAddressStructure {
    address: u64,
    address_type: u8,
}

#[derive(Debug)]
pub enum AcpiError {
    InvalidSignature,
    InvalidAddress,
    TableNotFound,
}

impl GeneralAddressStructure {
    pub const SPACE_ID_SYSTEM_MEMORY: u8 = 0x00;
    const SPACE_ID_INVALID: u8 = 0x0B;

    fn invalid() -> Self {
        Self {
            address: 0,
            address_type: Self::SPACE_ID_INVALID,
        }
    }

    pub fn new(a: &[u8; 12]) -> Self {
        let address_type = a[0];
        if address_type >= Self::SPACE_ID_INVALID {
            return Self::invalid();
        }
        Self {
            address_type,
            address: u64::from_le_bytes((a[4..12]).try_into().unwrap()),
        }
    }

    pub const fn is_invalid(&self) -> bool {
        self.address_type == Self::SPACE_ID_INVALID
    }

    pub const fn get_address_type(&self) -> u8 {
        self.address_type
    }

    pub const fn get_address(&self) -> u64 {
        self.address
    }
}

pub fn get_acpi_table(rsdp_address: usize, signature: &[u8; 4]) -> Result<usize, AcpiError> {
    let rsdp = unsafe { &*(rsdp_address as *const RSDP) };
    if rsdp.signature != RSDP_SIGNATURE {
        return Err(AcpiError::InvalidSignature);
    }
    if rsdp.xsdt_address == 0 {
        return Err(AcpiError::InvalidAddress);
    }
    let xsdt = unsafe { &*(rsdp.xsdt_address as *const XSDT) };

    if xsdt.signature != XSDT_SIGNATURE {
        return Err(AcpiError::InvalidSignature);
    }

    for table_index in 0..((xsdt.length as usize - XSDT_STRUCT_SIZE) >> 3) {
        let table_address = unsafe {
            *((rsdp.xsdt_address as usize + XSDT_STRUCT_SIZE + (table_index << 3)) as *const u64)
        } as usize;

        if unsafe { *(table_address as *const [u8; 4]) } == *signature {
            return Ok(table_address);
        }
    }

    return Err(AcpiError::TableNotFound);
}
