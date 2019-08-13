//! Common date/time function

use super::super::{BitFlags, DateTime, Error, Hours, Mcp794xx, Register, Rtcc};
use super::{decimal_to_packed_bcd, hours_from_register, hours_to_register, packed_bcd_to_decimal};
use interface;

impl<DI, E> Rtcc for Mcp794xx<DI>
where
    DI: interface::WriteData<Error = Error<E>> + interface::ReadData<Error = Error<E>>,
{
    type Error = Error<E>;

    fn get_seconds(&mut self) -> Result<u8, Self::Error> {
        let data = self.iface.read_register(Register::RTCSEC)?;
        let second = packed_bcd_to_decimal(data & !BitFlags::ST);
        Ok(second)
    }

    fn get_minutes(&mut self) -> Result<u8, Self::Error> {
        let data = self.iface.read_register(Register::RTCMIN)?;
        let minute = packed_bcd_to_decimal(data);
        Ok(minute)
    }

    fn get_hours(&mut self) -> Result<Hours, Self::Error> {
        let data = self.iface.read_register(Register::RTCHOUR)?;
        let hour = hours_from_register(data);
        Ok(hour)
    }

    fn get_weekday(&mut self) -> Result<u8, Self::Error> {
        let data = self.iface.read_register(Register::RTCWKDAY)?;
        let weekday = data & BitFlags::WKDAY;
        Ok(weekday)
    }

    fn get_day(&mut self) -> Result<u8, Self::Error> {
        let data = self.iface.read_register(Register::RTCDATE)?;
        let day = packed_bcd_to_decimal(data);
        Ok(day)
    }

    fn get_month(&mut self) -> Result<u8, Self::Error> {
        let data = self.iface.read_register(Register::RTCMONTH)?;
        let month = packed_bcd_to_decimal(data & !BitFlags::LPYR);
        Ok(month)
    }

    fn get_year(&mut self) -> Result<u16, Self::Error> {
        let data = self.iface.read_register(Register::RTCYEAR)?;
        let two_digit_year = packed_bcd_to_decimal(data);
        Ok(2000u16 + u16::from(two_digit_year))
    }

    fn set_seconds(&mut self, seconds: u8) -> Result<(), Self::Error> {
        Self::check_lt(seconds, 60)?;
        let second = decimal_to_packed_bcd(seconds);
        let value = if self.is_enabled {
            second | BitFlags::ST
        } else {
            second
        };
        self.iface.write_register(Register::RTCSEC, value)
    }

    fn set_minutes(&mut self, minutes: u8) -> Result<(), Self::Error> {
        Self::check_lt(minutes, 60)?;
        let data = decimal_to_packed_bcd(minutes);
        self.iface.write_register(Register::RTCMIN, data)
    }

    fn set_hours(&mut self, hours: Hours) -> Result<(), Self::Error> {
        let data = hours_to_register(hours)?;
        self.iface.write_register(Register::RTCHOUR, data)
    }

    fn set_weekday(&mut self, weekday: u8) -> Result<(), Self::Error> {
        Self::check_between(weekday, 1, 7)?;
        let mut data = self.iface.read_register(Register::RTCWKDAY)?;
        data &= !BitFlags::WKDAY;
        data |= decimal_to_packed_bcd(weekday);
        self.iface.write_register(Register::RTCWKDAY, data)
    }

    fn set_day(&mut self, day: u8) -> Result<(), Self::Error> {
        Self::check_between(day, 1, 31)?;
        let data = decimal_to_packed_bcd(day);
        self.iface.write_register(Register::RTCDATE, data)
    }

    fn set_month(&mut self, month: u8) -> Result<(), Self::Error> {
        Self::check_between(month, 1, 12)?;
        let data = decimal_to_packed_bcd(month);
        self.iface.write_register(Register::RTCMONTH, data)
    }

    fn set_year(&mut self, year: u16) -> Result<(), Self::Error> {
        Self::check_between(year, 2000, 2099)?;
        let data = decimal_to_packed_bcd((year - 2000) as u8);
        self.iface.write_register(Register::RTCYEAR, data)
    }

    fn get_datetime(&mut self) -> Result<DateTime, Self::Error> {
        let mut buffer: [u8; 8] = [0; 8];
        // Do a single read of all registers. The chip has an internal buffer
        // to avoid roll-over problems.
        buffer[0] = Register::RTCSEC;
        self.iface.read_data(&mut buffer)?;

        let second = packed_bcd_to_decimal(buffer[1] & !BitFlags::ST);
        let minute = packed_bcd_to_decimal(buffer[2]);
        let hour = hours_from_register(buffer[3]);
        let weekday = buffer[4] & BitFlags::WKDAY;
        let day = packed_bcd_to_decimal(buffer[5]);
        let month = packed_bcd_to_decimal(buffer[6] & !BitFlags::LPYR);
        let two_digit_year = packed_bcd_to_decimal(buffer[7]);
        let year = 2000u16 + u16::from(two_digit_year);

        Ok(DateTime {
            year,
            month,
            day,
            weekday,
            hour,
            minute,
            second,
        })
    }

    fn set_datetime(&mut self, datetime: &DateTime) -> Result<(), Self::Error> {
        let mut buffer: [u8; 8] = [0; 8];
        // Do a single read of all registers. The chip has an internal buffer
        // to avoid roll-over problems.
        buffer[0] = Register::RTCSEC;
        self.iface.read_data(&mut buffer)?;

        // Update the buffer with the new values
        Self::check_lt(datetime.second, 60)?;
        let second = decimal_to_packed_bcd(datetime.second);
        buffer[1] &= BitFlags::ST;
        buffer[1] |= second;
        Self::check_lt(datetime.minute, 60)?;
        buffer[2] = decimal_to_packed_bcd(datetime.minute);
        buffer[3] = hours_to_register(datetime.hour)?;
        Self::check_between(datetime.weekday, 1, 7)?;
        buffer[4] &= !BitFlags::WKDAY;
        buffer[4] |= decimal_to_packed_bcd(datetime.weekday);
        Self::check_between(datetime.day, 1, 31)?;
        buffer[5] = decimal_to_packed_bcd(datetime.day);
        Self::check_between(datetime.month, 1, 12)?;
        buffer[6] = decimal_to_packed_bcd(datetime.month);
        Self::check_between(datetime.year, 2000, 2099)?;
        buffer[7] = decimal_to_packed_bcd((datetime.year - 2000) as u8);

        self.iface.write_data(&buffer)?;
        Ok(())
    }
}
