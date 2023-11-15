use bitflags::bitflags;

bitflags! {
    #[derive(Debug)]
    pub struct CastlingFlags: u8 {
        const WK = 0b0000_0001;
        const WQ = 0b0000_0010;
        const BK = 0b0000_0100;
        const BQ = 0b0000_1000;

        const WHITE = Self::WK.bits() | Self::WQ.bits();
        const BLACK = Self::BK.bits() | Self::BQ.bits();
        const ALL   = Self::WHITE.bits() | Self::BLACK.bits();
    }
}