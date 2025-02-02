/// Stores colorspace information
pub trait Color: Sync + Send {
    /// The name of a colorspace, for example: "rgb"
    fn name() -> &'static str;

    /// The number of channels
    fn channels() -> usize;

    /// Determines if the last channel should be used as an alpha channel
    fn has_alpha() -> bool;
}

macro_rules! make_color {
    ($name:ident, $name_s:expr, $channels:expr, $alpha:expr) => {
        #[cfg_attr(feature = "ser", derive(serde::Serialize, serde::Deserialize))]
        #[derive(Debug, Clone, Copy, PartialEq)]
        pub struct $name;

        impl Color for $name {
            fn channels() -> usize {
                $channels
            }
            fn has_alpha() -> bool {
                $alpha
            }
            fn name() -> &'static str {
                $name_s
            }
        }
    };
}

make_color!(Gray, "gray", 1, false);

make_color!(Rgb, "rgb", 3, false);

make_color!(Bgr, "bgr", 3, false);

make_color!(RgbPacked, "rgb_packed", 1, false);

make_color!(Rgba, "rgba", 4, true);

make_color!(Bgra, "bgra", 4, true);

make_color!(RgbaPacked, "rgba_packed", 1, false);

make_color!(Cmyk, "cmyk", 4, false);

make_color!(Yuv, "yuv", 3, false);
