use super::{BitFlags, Error, Hours};
pub mod datetime;

// Transforms a decimal number to packed BCD format
pub(crate) fn decimal_to_packed_bcd(dec: u8) -> u8 {
    ((dec / 10) << 4) | (dec % 10)
}

// Transforms a number in packed BCD format to decimal
pub(crate) fn packed_bcd_to_decimal(bcd: u8) -> u8 {
    (bcd >> 4) * 10 + (bcd & 0xF)
}

pub(crate) fn hours_to_register<E>(hours: Hours) -> Result<u8, Error<E>> {
    match hours {
        Hours::H24(h) if h > 23 => Err(Error::InvalidInputData),
        Hours::H24(h) => Ok(decimal_to_packed_bcd(h)),
        Hours::AM(h) if h < 1 || h > 12 => Err(Error::InvalidInputData),
        Hours::AM(h) => Ok(BitFlags::H24_H12 | decimal_to_packed_bcd(h)),
        Hours::PM(h) if h < 1 || h > 12 => Err(Error::InvalidInputData),
        Hours::PM(h) => Ok(BitFlags::H24_H12 | BitFlags::AM_PM | decimal_to_packed_bcd(h)),
    }
}

pub(crate) fn hours_from_register(data: u8) -> Hours {
    if is_24h_format(data) {
        Hours::H24(packed_bcd_to_decimal(data & !BitFlags::H24_H12))
    } else if is_am(data) {
        Hours::AM(packed_bcd_to_decimal(
            data & !(BitFlags::H24_H12 | BitFlags::AM_PM),
        ))
    } else {
        Hours::PM(packed_bcd_to_decimal(
            data & !(BitFlags::H24_H12 | BitFlags::AM_PM),
        ))
    }
}

fn is_24h_format(hours_data: u8) -> bool {
    hours_data & BitFlags::H24_H12 == 0
}

fn is_am(hours_data: u8) -> bool {
    hours_data & BitFlags::AM_PM == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_convert_packed_bcd_to_decimal() {
        assert_eq!(0, packed_bcd_to_decimal(0b0000_0000));
        assert_eq!(1, packed_bcd_to_decimal(0b0000_0001));
        assert_eq!(9, packed_bcd_to_decimal(0b0000_1001));
        assert_eq!(10, packed_bcd_to_decimal(0b0001_0000));
        assert_eq!(11, packed_bcd_to_decimal(0b0001_0001));
        assert_eq!(19, packed_bcd_to_decimal(0b0001_1001));
        assert_eq!(20, packed_bcd_to_decimal(0b0010_0000));
        assert_eq!(21, packed_bcd_to_decimal(0b0010_0001));
        assert_eq!(59, packed_bcd_to_decimal(0b0101_1001));
    }

    #[test]
    fn can_convert_decimal_to_packed_bcd() {
        assert_eq!(0b0000_0000, decimal_to_packed_bcd(0));
        assert_eq!(0b0000_0001, decimal_to_packed_bcd(1));
        assert_eq!(0b0000_1001, decimal_to_packed_bcd(9));
        assert_eq!(0b0001_0000, decimal_to_packed_bcd(10));
        assert_eq!(0b0001_0001, decimal_to_packed_bcd(11));
        assert_eq!(0b0001_1001, decimal_to_packed_bcd(19));
        assert_eq!(0b0010_0000, decimal_to_packed_bcd(20));
        assert_eq!(0b0010_0001, decimal_to_packed_bcd(21));
        assert_eq!(0b0101_1001, decimal_to_packed_bcd(59));
    }
}
