#[macro_export]

macro_rules! set_sensitive {
    ($val:literal, $p:expr) => {
        $p.set_sensitive($val);
    };
    ($val:literal, $($p:ident).+{$($i:ident),+}) => {
        let s = &$($p).+;
        $(set_sensitive!($val, s.$i));+
    }
}