use bytemuck::{Pod, Zeroable};

use super::{interpolation::SpringPhysics, Interpolate};

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

unsafe impl Zeroable for Color {}
unsafe impl Pod for Color {}

impl Color {
    pub const DEFAULT: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };

    pub const fn from_rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const fn from_rgb(r: f32, g: f32, b: f32) -> Self {
        Self::from_rgba(r, g, b, 1.0)
    }

    pub const fn from_rgba8(r: u8, g: u8, b: u8, a: f32) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a,
        }
    }

    pub const fn from_rgb8(r: u8, g: u8, b: u8) -> Self {
        Self::from_rgba8(r, g, b, 1.0)
    }

    pub const fn with_alpha(mut self, a: f32) -> Self {
        self.a = a;
        self
    }

    pub const fn tint(mut self, amount: f32) -> Self {
        self.r = self.r + (1.0 - self.r) * amount;
        self.g = self.g + (1.0 - self.g) * amount;
        self.b = self.b + (1.0 - self.b) * amount;
        self
    }

    pub const fn shade(mut self, amount: f32) -> Self {
        self.r *= 1.0 - amount;
        self.g *= 1.0 - amount;
        self.b *= 1.0 - amount;
        self
    }
}

impl Interpolate for Color {
    fn lerp(&self, other: &Self, scalar: f64) -> Self {
        Self {
            r: self.r.lerp(&other.r, scalar).clamp(0.0, 1.0),
            g: self.g.lerp(&other.g, scalar).clamp(0.0, 1.0),
            b: self.b.lerp(&other.b, scalar).clamp(0.0, 1.0),
            a: self.a.lerp(&other.a, scalar).clamp(0.0, 1.0),
        }
    }
}

impl SpringPhysics for Color {
    const ZERO: Self = Self::DEFAULT;

    fn distance_squared_to(&self, other: &Self) -> f64 {
        self.r.distance_squared_to(&other.r)
            + self.g.distance_squared_to(&other.g)
            + self.b.distance_squared_to(&other.b)
            + self.a.distance_squared_to(&other.a)
    }

    fn apply_spring_update(
        &mut self,
        velocity: &mut Self,
        delta_t: f64,
        target: &Self,
        properties: &super::SpringProperties,
    ) {
        self.a
            .apply_spring_update(&mut velocity.a, delta_t, &target.a, properties);
        self.r
            .apply_spring_update(&mut velocity.r, delta_t, &target.r, properties);
        self.g
            .apply_spring_update(&mut velocity.g, delta_t, &target.g, properties);
        self.b
            .apply_spring_update(&mut velocity.b, delta_t, &target.b, properties);
        self.a = self.a.clamp(0.0, 1.0);
        self.r = self.r.clamp(0.0, 1.0);
        self.g = self.g.clamp(0.0, 1.0);
        self.b = self.b.clamp(0.0, 1.0);
    }
}

impl Color {
    pub const ABSOLUTE_ZERO: Self = Self::from_rgb8(0x00, 0x48, 0xba);
    pub const ACID_GREEN: Self = Self::from_rgb8(0xb0, 0xbf, 0x1a);
    pub const ALICEBLUE: Self = Self::from_rgb8(0xf0, 0xf8, 0xff);
    pub const ALIZARIN_CRIMSON: Self = Self::from_rgb8(0xe3, 0x26, 0x36);
    pub const AMARANTH: Self = Self::from_rgb8(0xe5, 0x2b, 0x50);
    pub const AMBER: Self = Self::from_rgb8(0xff, 0xbf, 0x00);
    pub const AMETHYST: Self = Self::from_rgb8(0x99, 0x66, 0xcc);
    pub const ANTIQUEWHITE: Self = Self::from_rgb8(0xfa, 0xeb, 0xd7);
    pub const APRICOT: Self = Self::from_rgb8(0xfb, 0xce, 0xb1);
    pub const AQUA: Self = Self::from_rgb8(0x00, 0xff, 0xff);
    pub const AQUA_MARINE: Self = Self::from_rgb8(0x7f, 0xff, 0xd4);
    pub const ARMY_GREEN: Self = Self::from_rgb8(0x4b, 0x53, 0x20);
    pub const ARYLIDE_YELLOW: Self = Self::from_rgb8(0xe9, 0xd6, 0x6b);
    pub const ASH_GREY: Self = Self::from_rgb8(0xb2, 0xbe, 0xb5);
    pub const ASPARAGUS: Self = Self::from_rgb8(0x87, 0xa9, 0x6b);
    pub const AUREOLIN: Self = Self::from_rgb8(0xfd, 0xee, 0x00);
    pub const AZURE: Self = Self::from_rgb8(0xf0, 0xff, 0xff);
    pub const BABY_BLUE: Self = Self::from_rgb8(0x89, 0xcf, 0xf0);
    pub const BABY_PINK: Self = Self::from_rgb8(0xf4, 0xc2, 0xc2);
    pub const BAKER_MILLER_PINK: Self = Self::from_rgb8(0xff, 0x91, 0xaf);
    pub const BANANA_MANIA: Self = Self::from_rgb8(0xfa, 0xe7, 0xb5);
    pub const BANANA_YELLOW: Self = Self::from_rgb8(0xff, 0xe1, 0x35);
    pub const BARN_RED: Self = Self::from_rgb8(0x7c, 0x0a, 0x02);
    pub const BATTLESHIP_GRAY: Self = Self::from_rgb8(0x84, 0x84, 0x82);
    pub const BEAVER: Self = Self::from_rgb8(0x9f, 0x81, 0x70);
    pub const BEIGE: Self = Self::from_rgb8(0xf5, 0xf5, 0xdc);
    pub const BISQUE: Self = Self::from_rgb8(0xff, 0xe4, 0xc4);
    pub const BISTRE: Self = Self::from_rgb8(0x3d, 0x2b, 0x1f);
    pub const BITTER_LEMON: Self = Self::from_rgb8(0xca, 0xe0, 0x0d);
    pub const BITTER_LIME: Self = Self::from_rgb8(0xbf, 0xff, 0x00);
    pub const BITTERSWEET: Self = Self::from_rgb8(0xfe, 0x6f, 0x5e);
    pub const BITTERSWEET_SHIMMER: Self = Self::from_rgb8(0xbf, 0x4f, 0x51);
    pub const BLACK: Self = Self::from_rgb8(0x00, 0x00, 0x00);
    pub const BLACK_COFFEE: Self = Self::from_rgb8(0x3b, 0x2f, 0x2f);
    pub const BLACK_OLIVE: Self = Self::from_rgb8(0x3b, 0x3c, 0x36);
    pub const BLACK_SHADOWS: Self = Self::from_rgb8(0xbf, 0xaf, 0xb2);
    pub const BLANCHEDALMOND: Self = Self::from_rgb8(0xff, 0xeb, 0xcd);
    pub const BLEU_DE_FRANCE: Self = Self::from_rgb8(0x31, 0x8c, 0xe7);
    pub const BLOND: Self = Self::from_rgb8(0xfa, 0xf0, 0xbe);
    pub const BLUE_PANTONE: Self = Self::from_rgb8(0x00, 0x18, 0xa8);
    pub const BLUE_BELL: Self = Self::from_rgb8(0xa2, 0xa2, 0xd0);
    pub const BLUE_GREEN: Self = Self::from_rgb8(0x0d, 0x98, 0xba);
    pub const BLUE: Self = Self::from_rgb8(0x00, 0x00, 0xff);
    pub const BLUEVIOLET: Self = Self::from_rgb8(0x8a, 0x2b, 0xe2);
    pub const BOLE: Self = Self::from_rgb8(0x79, 0x44, 0x3b);
    pub const BONE: Self = Self::from_rgb8(0xe3, 0xda, 0xc9);
    pub const BOYSENBERRY: Self = Self::from_rgb8(0x87, 0x32, 0x60);
    pub const BRANDEIS_BLUE: Self = Self::from_rgb8(0x00, 0x70, 0xff);
    pub const BRASS: Self = Self::from_rgb8(0xb5, 0xa6, 0x42);
    pub const BRICK_RED: Self = Self::from_rgb8(0xcb, 0x41, 0x54);
    pub const BRIGHT_CERULEAN: Self = Self::from_rgb8(0x1d, 0xac, 0xd6);
    pub const BRIGHT_GREEN: Self = Self::from_rgb8(0x66, 0xff, 0x00);
    pub const BRIGHT_LAVENDER: Self = Self::from_rgb8(0xbf, 0x94, 0xe4);
    pub const BRIGHT_LILAC: Self = Self::from_rgb8(0xd8, 0x91, 0xef);
    pub const BRIGHT_MAROON: Self = Self::from_rgb8(0xc3, 0x21, 0x48);
    pub const BRIGHT_NAVY_BLUE: Self = Self::from_rgb8(0x19, 0x74, 0xd2);
    pub const BRIGHT_TURQUOISE: Self = Self::from_rgb8(0x08, 0xe8, 0xde);
    pub const BRIGHT_UBE: Self = Self::from_rgb8(0xd1, 0x9f, 0xe8);
    pub const BRILLIANT_ROSE: Self = Self::from_rgb8(0xff, 0x55, 0xa3);
    pub const BRINK_PINK: Self = Self::from_rgb8(0xfb, 0x60, 0x7f);
    pub const BRITISH_RACING_GREEN: Self = Self::from_rgb8(0x00, 0x42, 0x25);
    pub const BRONZE: Self = Self::from_rgb8(0xcd, 0x7f, 0x32);
    pub const BROWN: Self = Self::from_rgb8(0xa5, 0x2a, 0x2a);
    pub const BRUNSWICK_GREEN: Self = Self::from_rgb8(0x1b, 0x4d, 0x3e);
    pub const BUBBLE_GUM: Self = Self::from_rgb8(0xff, 0xc1, 0xcc);
    pub const BUFF: Self = Self::from_rgb8(0xf0, 0xdc, 0x82);
    pub const BULGARIAN_ROSE: Self = Self::from_rgb8(0x48, 0x06, 0x07);
    pub const BURGUNDY: Self = Self::from_rgb8(0x80, 0x00, 0x20);
    pub const BURLYWOOD: Self = Self::from_rgb8(0xde, 0xb8, 0x87);
    pub const BURNISHED_BROWN: Self = Self::from_rgb8(0xa1, 0x7a, 0x74);
    pub const BURNT_ORANGE: Self = Self::from_rgb8(0xcc, 0x55, 0x00);
    pub const BURNT_SIENNA: Self = Self::from_rgb8(0xe9, 0x74, 0x51);
    pub const BURNT_UMBER: Self = Self::from_rgb8(0x8a, 0x33, 0x24);
    pub const BYZANTINE: Self = Self::from_rgb8(0xbd, 0x33, 0xa4);
    pub const BYZANTIUM: Self = Self::from_rgb8(0x70, 0x29, 0x63);
    pub const CADET_GREY: Self = Self::from_rgb8(0x91, 0xa3, 0xb0);
    pub const CADETBLUE: Self = Self::from_rgb8(0x5f, 0x9e, 0xa0);
    pub const CADMIUM_GREEN: Self = Self::from_rgb8(0x00, 0x6b, 0x3c);
    pub const CADMIUM_ORANGE: Self = Self::from_rgb8(0xed, 0x87, 0x2d);
    pub const CADMIUM_RED: Self = Self::from_rgb8(0xe3, 0x00, 0x22);
    pub const CADMIUM_YELLOW: Self = Self::from_rgb8(0xff, 0xf6, 0x00);
    pub const CAMBRIDGE_BLUE: Self = Self::from_rgb8(0xa3, 0xc1, 0xad);
    pub const CAMEL: Self = Self::from_rgb8(0xc1, 0x9a, 0x6b);
    pub const CAMEO_PINK: Self = Self::from_rgb8(0xef, 0xbb, 0xcc);
    pub const CAMOUFLAGE_GREEN: Self = Self::from_rgb8(0x78, 0x86, 0x6b);
    pub const CANARY: Self = Self::from_rgb8(0xff, 0xff, 0x99);
    pub const CANARY_YELLOW: Self = Self::from_rgb8(0xff, 0xef, 0x00);
    pub const CANDY_APPLE_RED: Self = Self::from_rgb8(0xff, 0x08, 0x00);
    pub const CANDY_PINK: Self = Self::from_rgb8(0xe4, 0x71, 0x7a);
    pub const CAPUT_MORTUUM: Self = Self::from_rgb8(0x59, 0x27, 0x20);
    pub const CARDINAL: Self = Self::from_rgb8(0xc4, 0x1e, 0x3a);
    pub const CARIBBEAN_GREEN: Self = Self::from_rgb8(0x00, 0xcc, 0x99);
    pub const CARMINE: Self = Self::from_rgb8(0x96, 0x00, 0x18);
    pub const CARMINE_PINK: Self = Self::from_rgb8(0xeb, 0x4c, 0x42);
    pub const CARNATION_PINK: Self = Self::from_rgb8(0xff, 0xa6, 0xc9);
    pub const CARNELIAN: Self = Self::from_rgb8(0xb3, 0x1b, 0x1b);
    pub const CAROLINA_BLUE: Self = Self::from_rgb8(0x56, 0xa0, 0xd3);
    pub const CARROT_ORANGE: Self = Self::from_rgb8(0xed, 0x91, 0x21);
    pub const CASTLETON_GREEN: Self = Self::from_rgb8(0x00, 0x56, 0x3f);
    pub const CEDAR_CHEST: Self = Self::from_rgb8(0xc9, 0x5a, 0x49);
    pub const CELADON: Self = Self::from_rgb8(0xac, 0xe1, 0xaf);
    pub const CELADON_GREEN: Self = Self::from_rgb8(0x2f, 0x84, 0x7c);
    pub const CELESTE: Self = Self::from_rgb8(0xb2, 0xff, 0xff);
    pub const CELTIC_BLUE: Self = Self::from_rgb8(0x24, 0x6b, 0xce);
    pub const CERISE: Self = Self::from_rgb8(0xde, 0x31, 0x63);
    pub const CERULEAN: Self = Self::from_rgb8(0x00, 0x7b, 0xa7);
    pub const CERULEAN_BLUE: Self = Self::from_rgb8(0x2a, 0x52, 0xbe);
    pub const CERULEAN_FROST: Self = Self::from_rgb8(0x6d, 0x9b, 0xc3);
    pub const CG_BLUE: Self = Self::from_rgb8(0x00, 0x7a, 0xa5);
    pub const CHAMOISEE: Self = Self::from_rgb8(0xa0, 0x78, 0x5a);
    pub const CHAMPAGNE: Self = Self::from_rgb8(0xf7, 0xe7, 0xce);
    pub const CHARCOAL: Self = Self::from_rgb8(0x36, 0x45, 0x4f);
    pub const CHARTREUSE: Self = Self::from_rgb8(0x7f, 0xff, 0x00);
    pub const CHERRY_BLOSSOM_PINK: Self = Self::from_rgb8(0xff, 0xb7, 0xc5);
    pub const CHESTNUT: Self = Self::from_rgb8(0x95, 0x45, 0x35);
    pub const CHOCOLATE: Self = Self::from_rgb8(0xd2, 0x69, 0x1e);
    pub const CHROME_YELLOW: Self = Self::from_rgb8(0xff, 0xa7, 0x00);
    pub const CINEREOUS: Self = Self::from_rgb8(0x98, 0x81, 0x7b);
    pub const CINNABAR: Self = Self::from_rgb8(0xe3, 0x42, 0x34);
    pub const CITRINE: Self = Self::from_rgb8(0xe4, 0xd0, 0x0a);
    pub const CITRON: Self = Self::from_rgb8(0x9f, 0xa9, 0x1f);
    pub const CLARET: Self = Self::from_rgb8(0x7f, 0x17, 0x34);
    pub const COBALT: Self = Self::from_rgb8(0x00, 0x47, 0xab);
    pub const COFFEE: Self = Self::from_rgb8(0x6f, 0x4e, 0x37);
    pub const COOL_GREY: Self = Self::from_rgb8(0x8c, 0x92, 0xac);
    pub const COPPER: Self = Self::from_rgb8(0xb8, 0x73, 0x33);
    pub const COPPER_RED: Self = Self::from_rgb8(0xcb, 0x6d, 0x51);
    pub const COPPER_ROSE: Self = Self::from_rgb8(0x99, 0x66, 0x66);
    pub const COQUELICOT: Self = Self::from_rgb8(0xff, 0x38, 0x00);
    pub const CORAL: Self = Self::from_rgb8(0xff, 0x7f, 0x50);
    pub const CORAL_PINK: Self = Self::from_rgb8(0xf8, 0x83, 0x79);
    pub const CORDOVAN: Self = Self::from_rgb8(0x89, 0x3f, 0x45);
    pub const CORN: Self = Self::from_rgb8(0xfb, 0xec, 0x5d);
    pub const CORNFLOWERBLUE: Self = Self::from_rgb8(0x64, 0x95, 0xed);
    pub const CORN_SILK: Self = Self::from_rgb8(0xff, 0xf8, 0xdc);
    pub const COSMIC_COBALT: Self = Self::from_rgb8(0x2e, 0x2d, 0x88);
    pub const COSMIC_LATTE: Self = Self::from_rgb8(0xff, 0xf8, 0xe7);
    pub const COTTON_CANDY: Self = Self::from_rgb8(0xff, 0xbc, 0xd9);
    pub const CREAM: Self = Self::from_rgb8(0xff, 0xfd, 0xd0);
    pub const CRIMSON: Self = Self::from_rgb8(0xdc, 0x14, 0x3c);
    pub const CRYSTAL: Self = Self::from_rgb8(0xa7, 0xd8, 0xde);
    pub const CYAN: Self = Self::from_rgb8(0x00, 0xff, 0xff);
    pub const CYCLAMEN: Self = Self::from_rgb8(0xf5, 0x6f, 0xa1);
    pub const DAFFODIL: Self = Self::from_rgb8(0xff, 0xff, 0x31);
    pub const DANDELION: Self = Self::from_rgb8(0xf0, 0xe1, 0x30);
    pub const DARK_BROWN: Self = Self::from_rgb8(0x65, 0x43, 0x21);
    pub const DARK_BYZANTIUM: Self = Self::from_rgb8(0x5d, 0x39, 0x54);
    pub const DARK_JUNGLE_GREEN: Self = Self::from_rgb8(0x1a, 0x24, 0x21);
    pub const DARK_LAVENDER: Self = Self::from_rgb8(0x73, 0x4f, 0x96);
    pub const DARK_MOSS_GREEN: Self = Self::from_rgb8(0x4a, 0x5d, 0x23);
    pub const DARK_PASTEL_GREEN: Self = Self::from_rgb8(0x03, 0xc0, 0x3c);
    pub const DARK_SIENNA: Self = Self::from_rgb8(0x3c, 0x14, 0x14);
    pub const DARK_SKY_BLUE: Self = Self::from_rgb8(0x8c, 0xbe, 0xd6);
    pub const DARK_SPRING_GREEN: Self = Self::from_rgb8(0x17, 0x72, 0x45);
    pub const DARK_GOLDEN_ROD: Self = Self::from_rgb8(0xb8, 0x86, 0x0b);
    pub const DARKGREEN: Self = Self::from_rgb8(0x00, 0x64, 0x00);
    pub const DARKKHAKI: Self = Self::from_rgb8(0xbd, 0xb7, 0x6b);
    pub const DARK_OLIVE_GREEN: Self = Self::from_rgb8(0x55, 0x6b, 0x2f);
    pub const DARK_ORANGE: Self = Self::from_rgb8(0xff, 0x8c, 0x00);
    pub const DARK_ORCHID: Self = Self::from_rgb8(0x99, 0x32, 0xcc);
    pub const DARKSALMON: Self = Self::from_rgb8(0xe9, 0x96, 0x7a);
    pub const DARKSEAGREEN: Self = Self::from_rgb8(0x8f, 0xbc, 0x8f);
    pub const DARKSLATEBLUE: Self = Self::from_rgb8(0x48, 0x3d, 0x8b);
    pub const DARK_SLATE_GRAY: Self = Self::from_rgb8(0x2f, 0x4f, 0x4f);
    pub const DARKTURQUOISE: Self = Self::from_rgb8(0x00, 0xce, 0xd1);
    pub const DARKVIOLET: Self = Self::from_rgb8(0x94, 0x00, 0xd3);
    pub const DARTMOUTH_GREEN: Self = Self::from_rgb8(0x00, 0x70, 0x3c);
    pub const DEEP_CERISE: Self = Self::from_rgb8(0xda, 0x32, 0x87);
    pub const DEEP_CHAMPAGNE: Self = Self::from_rgb8(0xfa, 0xd6, 0xa5);
    pub const DEEP_FUCHSIA: Self = Self::from_rgb8(0xc1, 0x54, 0xc1);
    pub const DEEP_JUNGLE_GREEN: Self = Self::from_rgb8(0x00, 0x4b, 0x49);
    pub const DEEP_PEACH: Self = Self::from_rgb8(0xff, 0xcb, 0xa4);
    pub const DEEP_SAFFRON: Self = Self::from_rgb8(0xff, 0x99, 0x33);
    pub const DEEP_SPACE_SPARKLE: Self = Self::from_rgb8(0x4a, 0x64, 0x6c);
    pub const DEEP_CHESTNUT: Self = Self::from_rgb8(0xb9, 0x4e, 0x48);
    pub const DEEP_PINK: Self = Self::from_rgb8(0xff, 0x14, 0x93);
    pub const DEEP_SKY_BLUE: Self = Self::from_rgb8(0x00, 0xbf, 0xff);
    pub const DENIM: Self = Self::from_rgb8(0x15, 0x60, 0xbd);
    pub const DENIM_BLUE: Self = Self::from_rgb8(0x22, 0x43, 0xb6);
    pub const DESERT_SAND: Self = Self::from_rgb8(0xed, 0xc9, 0xaf);
    pub const DIMGRAY: Self = Self::from_rgb8(0x69, 0x69, 0x69);
    pub const DODGERBLUE: Self = Self::from_rgb8(0x1e, 0x90, 0xff);
    pub const DOGWOOD_ROSE: Self = Self::from_rgb8(0xd7, 0x18, 0x68);
    pub const DUTCH_WHITE: Self = Self::from_rgb8(0xef, 0xdf, 0xbb);
    pub const EARTH_YELLOW: Self = Self::from_rgb8(0xe1, 0xa9, 0x5f);
    pub const EBONY: Self = Self::from_rgb8(0x55, 0x5d, 0x50);
    pub const EGGPLANT: Self = Self::from_rgb8(0x61, 0x40, 0x51);
    pub const EGGSHELL: Self = Self::from_rgb8(0xf0, 0xea, 0xd6);
    pub const EGYPTIAN_BLUE: Self = Self::from_rgb8(0x10, 0x34, 0xa6);
    pub const ELECTRIC_BLUE: Self = Self::from_rgb8(0x7d, 0xf9, 0xff);
    pub const ELECTRIC_INDIGO: Self = Self::from_rgb8(0x6f, 0x00, 0xff);
    pub const ELECTRIC_LIME: Self = Self::from_rgb8(0xcc, 0xff, 0x00);
    pub const ELECTRIC_PURPLE: Self = Self::from_rgb8(0xbf, 0x00, 0xff);
    pub const EMERALD: Self = Self::from_rgb8(0x50, 0xc8, 0x78);
    pub const EMINENCE: Self = Self::from_rgb8(0x6c, 0x30, 0x82);
    pub const ETON_BLUE: Self = Self::from_rgb8(0x96, 0xc8, 0xa2);
    pub const FALU_RED: Self = Self::from_rgb8(0x80, 0x18, 0x18);
    pub const FAWN: Self = Self::from_rgb8(0xe5, 0xaa, 0x70);
    pub const FELDGRAU: Self = Self::from_rgb8(0x4d, 0x5d, 0x53);
    pub const FERN_GREEN: Self = Self::from_rgb8(0x4f, 0x79, 0x42);
    pub const FERRARI_RED: Self = Self::from_rgb8(0xff, 0x28, 0x00);
    pub const FIRE_OPAL: Self = Self::from_rgb8(0xe9, 0x5c, 0x4b);
    pub const FIREBRICK: Self = Self::from_rgb8(0xb2, 0x22, 0x22);
    pub const FLAMINGO_PINK: Self = Self::from_rgb8(0xfc, 0x8e, 0xac);
    pub const FLORALWHITE: Self = Self::from_rgb8(0xff, 0xfa, 0xf0);
    pub const FLOURESCENT_BLUE: Self = Self::from_rgb8(0x15, 0xf4, 0xee);
    pub const FOREST_GREEN: Self = Self::from_rgb8(0x22, 0x8b, 0x22);
    pub const FORESTGREEN: Self = Self::from_rgb8(0x22, 0x8b, 0x22);
    pub const FRENCH_BEIGE: Self = Self::from_rgb8(0xa6, 0x7b, 0x5b);
    pub const FRENCH_BISTRE: Self = Self::from_rgb8(0x85, 0x6d, 0x4d);
    pub const FRENCH_BLUE: Self = Self::from_rgb8(0x00, 0x72, 0xbb);
    pub const FRENCH_LILAC: Self = Self::from_rgb8(0x86, 0x60, 0x8e);
    pub const FRENCH_MAUVE: Self = Self::from_rgb8(0xd4, 0x73, 0xd4);
    pub const FRENCH_PINK: Self = Self::from_rgb8(0xfd, 0x6c, 0x9e);
    pub const FRENCH_ROSE: Self = Self::from_rgb8(0xf6, 0x4a, 0x8a);
    pub const FRENCH_SKY_BLUE: Self = Self::from_rgb8(0x77, 0xb5, 0xfe);
    pub const FRENCH_VIOLET: Self = Self::from_rgb8(0x88, 0x06, 0xce);
    pub const FROSTBITE: Self = Self::from_rgb8(0xe9, 0x36, 0xa7);
    pub const FUCHSIA_PURPLE: Self = Self::from_rgb8(0xcc, 0x39, 0x7b);
    pub const FUCHSIA_ROSE: Self = Self::from_rgb8(0xc7, 0x43, 0x75);
    pub const FULVOUS: Self = Self::from_rgb8(0xe4, 0x84, 0x00);
    pub const FUZZY_WUZZY: Self = Self::from_rgb8(0x87, 0x42, 0x1f);
    pub const GO_GREEN: Self = Self::from_rgb8(0x00, 0xab, 0x66);
    pub const GAINSBORO: Self = Self::from_rgb8(0xdc, 0xdc, 0xdc);
    pub const GAMBOGE: Self = Self::from_rgb8(0xe4, 0x9b, 0x0f);
    pub const GENERIC_VIRIDIAN: Self = Self::from_rgb8(0x00, 0x7f, 0x66);
    pub const GHOSTWHITE: Self = Self::from_rgb8(0xf8, 0xf8, 0xff);
    pub const GINGER: Self = Self::from_rgb8(0xb0, 0x65, 0x00);
    pub const GLAUCOUS: Self = Self::from_rgb8(0x60, 0x82, 0xb6);
    pub const GLOSSY_GRAPE: Self = Self::from_rgb8(0xab, 0x92, 0xb3);
    pub const GOLD_FUSION: Self = Self::from_rgb8(0x85, 0x75, 0x4e);
    pub const GOLD: Self = Self::from_rgb8(0xff, 0xd7, 0x00);
    pub const GOLDEN_BROWN: Self = Self::from_rgb8(0x99, 0x65, 0x15);
    pub const GOLDEN_POPPY: Self = Self::from_rgb8(0xfc, 0xc2, 0x00);
    pub const GOLDEN_YELLOW: Self = Self::from_rgb8(0xff, 0xdf, 0x00);
    pub const GOLDENROD: Self = Self::from_rgb8(0xda, 0xa5, 0x20);
    pub const GRANITE_GRAY: Self = Self::from_rgb8(0x67, 0x67, 0x67);
    pub const GRANNY_SMITH_APPLE: Self = Self::from_rgb8(0xa8, 0xe4, 0xa0);
    pub const GRAY: Self = Self::from_rgb8(0xbe, 0xbe, 0xbe);
    pub const GRAY1: Self = Self::from_rgb8(0x03, 0x03, 0x03);
    pub const GRAY10: Self = Self::from_rgb8(0x1a, 0x1a, 0x1a);
    pub const GRAY11: Self = Self::from_rgb8(0x1c, 0x1c, 0x1c);
    pub const GRAY12: Self = Self::from_rgb8(0x1f, 0x1f, 0x1f);
    pub const GRAY13: Self = Self::from_rgb8(0x21, 0x21, 0x21);
    pub const GRAY14: Self = Self::from_rgb8(0x24, 0x24, 0x24);
    pub const GRAY15: Self = Self::from_rgb8(0x26, 0x26, 0x26);
    pub const GRAY16: Self = Self::from_rgb8(0x29, 0x29, 0x29);
    pub const GRAY17: Self = Self::from_rgb8(0x2b, 0x2b, 0x2b);
    pub const GRAY18: Self = Self::from_rgb8(0x2e, 0x2e, 0x2e);
    pub const GRAY19: Self = Self::from_rgb8(0x30, 0x30, 0x30);
    pub const GRAY2: Self = Self::from_rgb8(0x05, 0x05, 0x05);
    pub const GRAY20: Self = Self::from_rgb8(0x33, 0x33, 0x33);
    pub const GRAY21: Self = Self::from_rgb8(0x36, 0x36, 0x36);
    pub const GRAY22: Self = Self::from_rgb8(0x38, 0x38, 0x38);
    pub const GRAY23: Self = Self::from_rgb8(0x3b, 0x3b, 0x3b);
    pub const GRAY24: Self = Self::from_rgb8(0x3d, 0x3d, 0x3d);
    pub const GRAY25: Self = Self::from_rgb8(0x40, 0x40, 0x40);
    pub const GRAY26: Self = Self::from_rgb8(0x42, 0x42, 0x42);
    pub const GRAY27: Self = Self::from_rgb8(0x45, 0x45, 0x45);
    pub const GRAY28: Self = Self::from_rgb8(0x47, 0x47, 0x47);
    pub const GRAY29: Self = Self::from_rgb8(0x4a, 0x4a, 0x4a);
    pub const GRAY3: Self = Self::from_rgb8(0x08, 0x08, 0x08);
    pub const GRAY30: Self = Self::from_rgb8(0x4d, 0x4d, 0x4d);
    pub const GRAY31: Self = Self::from_rgb8(0x4f, 0x4f, 0x4f);
    pub const GRAY32: Self = Self::from_rgb8(0x52, 0x52, 0x52);
    pub const GRAY33: Self = Self::from_rgb8(0x54, 0x54, 0x54);
    pub const GRAY34: Self = Self::from_rgb8(0x57, 0x57, 0x57);
    pub const GRAY35: Self = Self::from_rgb8(0x59, 0x59, 0x59);
    pub const GRAY36: Self = Self::from_rgb8(0x5c, 0x5c, 0x5c);
    pub const GRAY37: Self = Self::from_rgb8(0x5e, 0x5e, 0x5e);
    pub const GRAY38: Self = Self::from_rgb8(0x61, 0x61, 0x61);
    pub const GRAY39: Self = Self::from_rgb8(0x63, 0x63, 0x63);
    pub const GRAY4: Self = Self::from_rgb8(0x0a, 0x0a, 0x0a);
    pub const GRAY40: Self = Self::from_rgb8(0x66, 0x66, 0x66);
    pub const GRAY41: Self = Self::from_rgb8(0x69, 0x69, 0x69);
    pub const GRAY42: Self = Self::from_rgb8(0x6b, 0x6b, 0x6b);
    pub const GRAY43: Self = Self::from_rgb8(0x6e, 0x6e, 0x6e);
    pub const GRAY44: Self = Self::from_rgb8(0x70, 0x70, 0x70);
    pub const GRAY45: Self = Self::from_rgb8(0x73, 0x73, 0x73);
    pub const GRAY46: Self = Self::from_rgb8(0x75, 0x75, 0x75);
    pub const GRAY47: Self = Self::from_rgb8(0x78, 0x78, 0x78);
    pub const GRAY48: Self = Self::from_rgb8(0x7a, 0x7a, 0x7a);
    pub const GRAY49: Self = Self::from_rgb8(0x7d, 0x7d, 0x7d);
    pub const GRAY5: Self = Self::from_rgb8(0x0d, 0x0d, 0x0d);
    pub const GRAY50: Self = Self::from_rgb8(0x7f, 0x7f, 0x7f);
    pub const GRAY51: Self = Self::from_rgb8(0x82, 0x82, 0x82);
    pub const GRAY52: Self = Self::from_rgb8(0x85, 0x85, 0x85);
    pub const GRAY53: Self = Self::from_rgb8(0x87, 0x87, 0x87);
    pub const GRAY54: Self = Self::from_rgb8(0x8a, 0x8a, 0x8a);
    pub const GRAY55: Self = Self::from_rgb8(0x8c, 0x8c, 0x8c);
    pub const GRAY56: Self = Self::from_rgb8(0x8f, 0x8f, 0x8f);
    pub const GRAY57: Self = Self::from_rgb8(0x91, 0x91, 0x91);
    pub const GRAY58: Self = Self::from_rgb8(0x94, 0x94, 0x94);
    pub const GRAY59: Self = Self::from_rgb8(0x96, 0x96, 0x96);
    pub const GRAY6: Self = Self::from_rgb8(0x0f, 0x0f, 0x0f);
    pub const GRAY60: Self = Self::from_rgb8(0x99, 0x99, 0x99);
    pub const GRAY61: Self = Self::from_rgb8(0x9c, 0x9c, 0x9c);
    pub const GRAY62: Self = Self::from_rgb8(0x9e, 0x9e, 0x9e);
    pub const GRAY63: Self = Self::from_rgb8(0xa1, 0xa1, 0xa1);
    pub const GRAY64: Self = Self::from_rgb8(0xa3, 0xa3, 0xa3);
    pub const GRAY65: Self = Self::from_rgb8(0xa6, 0xa6, 0xa6);
    pub const GRAY66: Self = Self::from_rgb8(0xa8, 0xa8, 0xa8);
    pub const GRAY67: Self = Self::from_rgb8(0xab, 0xab, 0xab);
    pub const GRAY68: Self = Self::from_rgb8(0xad, 0xad, 0xad);
    pub const GRAY69: Self = Self::from_rgb8(0xb0, 0xb0, 0xb0);
    pub const GRAY7: Self = Self::from_rgb8(0x12, 0x12, 0x12);
    pub const GRAY70: Self = Self::from_rgb8(0xb3, 0xb3, 0xb3);
    pub const GRAY71: Self = Self::from_rgb8(0xb5, 0xb5, 0xb5);
    pub const GRAY72: Self = Self::from_rgb8(0xb8, 0xb8, 0xb8);
    pub const GRAY73: Self = Self::from_rgb8(0xba, 0xba, 0xba);
    pub const GRAY74: Self = Self::from_rgb8(0xbd, 0xbd, 0xbd);
    pub const GRAY75: Self = Self::from_rgb8(0xbf, 0xbf, 0xbf);
    pub const GRAY76: Self = Self::from_rgb8(0xc2, 0xc2, 0xc2);
    pub const GRAY77: Self = Self::from_rgb8(0xc4, 0xc4, 0xc4);
    pub const GRAY78: Self = Self::from_rgb8(0xc7, 0xc7, 0xc7);
    pub const GRAY79: Self = Self::from_rgb8(0xc9, 0xc9, 0xc9);
    pub const GRAY8: Self = Self::from_rgb8(0x14, 0x14, 0x14);
    pub const GRAY80: Self = Self::from_rgb8(0xcc, 0xcc, 0xcc);
    pub const GRAY81: Self = Self::from_rgb8(0xcf, 0xcf, 0xcf);
    pub const GRAY82: Self = Self::from_rgb8(0xd1, 0xd1, 0xd1);
    pub const GRAY83: Self = Self::from_rgb8(0xd4, 0xd4, 0xd4);
    pub const GRAY84: Self = Self::from_rgb8(0xd6, 0xd6, 0xd6);
    pub const GRAY85: Self = Self::from_rgb8(0xd9, 0xd9, 0xd9);
    pub const GRAY86: Self = Self::from_rgb8(0xdb, 0xdb, 0xdb);
    pub const GRAY87: Self = Self::from_rgb8(0xde, 0xde, 0xde);
    pub const GRAY88: Self = Self::from_rgb8(0xe0, 0xe0, 0xe0);
    pub const GRAY89: Self = Self::from_rgb8(0xe3, 0xe3, 0xe3);
    pub const GRAY9: Self = Self::from_rgb8(0x17, 0x17, 0x17);
    pub const GRAY90: Self = Self::from_rgb8(0xe5, 0xe5, 0xe5);
    pub const GRAY91: Self = Self::from_rgb8(0xe8, 0xe8, 0xe8);
    pub const GRAY92: Self = Self::from_rgb8(0xeb, 0xeb, 0xeb);
    pub const GRAY93: Self = Self::from_rgb8(0xed, 0xed, 0xed);
    pub const GRAY94: Self = Self::from_rgb8(0xf0, 0xf0, 0xf0);
    pub const GRAY95: Self = Self::from_rgb8(0xf2, 0xf2, 0xf2);
    pub const GRAY97: Self = Self::from_rgb8(0xf7, 0xf7, 0xf7);
    pub const GRAY98: Self = Self::from_rgb8(0xfa, 0xfa, 0xfa);
    pub const GRAY99: Self = Self::from_rgb8(0xfc, 0xfc, 0xfc);
    pub const GREEN_CRAYOLA: Self = Self::from_rgb8(0x1c, 0xac, 0x78);
    pub const GREEN_PANTONE: Self = Self::from_rgb8(0x00, 0xad, 0x43);
    pub const GREEN_PIGMENT: Self = Self::from_rgb8(0x00, 0xa5, 0x50);
    pub const GREEN_LIZARD: Self = Self::from_rgb8(0xa7, 0xf4, 0x32);
    pub const GREEN_SHEEN: Self = Self::from_rgb8(0x6e, 0xae, 0xa1);
    pub const GREEN: Self = Self::from_rgb8(0x00, 0xff, 0x00);
    pub const GREEN_YELLOW: Self = Self::from_rgb8(0xad, 0xff, 0x2f);
    pub const GRULLO: Self = Self::from_rgb8(0xa9, 0x9a, 0x86);
    pub const GUNMETAL: Self = Self::from_rgb8(0x2a, 0x34, 0x39);
    pub const HAN_BLUE: Self = Self::from_rgb8(0x44, 0x6c, 0xcf);
    pub const HAN_PURPLE: Self = Self::from_rgb8(0x52, 0x18, 0xfa);
    pub const HARLEQUIN: Self = Self::from_rgb8(0x3f, 0xff, 0x00);
    pub const HARVEST_GOLD: Self = Self::from_rgb8(0xda, 0x91, 0x00);
    pub const HELIOTROPE: Self = Self::from_rgb8(0xdf, 0x73, 0xff);
    pub const HOLLYWOOD_CERISE: Self = Self::from_rgb8(0xf4, 0x00, 0xa1);
    pub const HONEY_DEW: Self = Self::from_rgb8(0xf0, 0xff, 0xf0);
    pub const HONOLULU_BLUE: Self = Self::from_rgb8(0x00, 0x6d, 0xb0);
    pub const HOT_MAGENTA: Self = Self::from_rgb8(0xff, 0x1d, 0xce);
    pub const HOTPINK: Self = Self::from_rgb8(0xff, 0x69, 0xb4);
    pub const HUNTER_GREEN: Self = Self::from_rgb8(0x35, 0x5e, 0x3b);
    pub const ICEBERG: Self = Self::from_rgb8(0x71, 0xa6, 0xd2);
    pub const ICTERINE: Self = Self::from_rgb8(0xfc, 0xf7, 0x5e);
    pub const ILLUMINATING_EMERALD: Self = Self::from_rgb8(0x31, 0x91, 0x77);
    pub const IMPERIAL_RED: Self = Self::from_rgb8(0xed, 0x29, 0x39);
    pub const INCHWORM: Self = Self::from_rgb8(0xb2, 0xec, 0x5d);
    pub const INDIA_GREEN: Self = Self::from_rgb8(0x13, 0x88, 0x08);
    pub const INDIAN_YELLOW: Self = Self::from_rgb8(0xe3, 0xa8, 0x57);
    pub const INDIANRED: Self = Self::from_rgb8(0xcd, 0x5c, 0x5c);
    pub const INDIGO: Self = Self::from_rgb8(0x4b, 0x00, 0x82);
    pub const INTERNATIONAL_ORANGE: Self = Self::from_rgb8(0xff, 0x4f, 0x00);
    pub const IRIS: Self = Self::from_rgb8(0x5a, 0x4f, 0xcf);
    pub const ISABELLINE: Self = Self::from_rgb8(0xf4, 0xf0, 0xec);
    pub const IVORY: Self = Self::from_rgb8(0xff, 0xff, 0xf0);
    pub const JADE: Self = Self::from_rgb8(0x00, 0xa8, 0x6b);
    pub const JAPANESE_CARMINE: Self = Self::from_rgb8(0x9d, 0x29, 0x33);
    pub const JASMINE: Self = Self::from_rgb8(0xf8, 0xde, 0x7e);
    pub const JAZZBERRY_JAM: Self = Self::from_rgb8(0xa5, 0x0b, 0x5e);
    pub const JONQUIL: Self = Self::from_rgb8(0xf4, 0xca, 0x16);
    pub const JUNGLE_GREEN: Self = Self::from_rgb8(0x29, 0xab, 0x87);
    pub const KELLY_GREEN: Self = Self::from_rgb8(0x4c, 0xbb, 0x17);
    pub const KEPPEL: Self = Self::from_rgb8(0x3a, 0xb0, 0x9e);
    pub const KEY_LIME: Self = Self::from_rgb8(0xe8, 0xf4, 0x8c);
    pub const KHAKI: Self = Self::from_rgb8(0xf0, 0xe6, 0x8c);
    pub const KOMBU_GREEN: Self = Self::from_rgb8(0x35, 0x42, 0x30);
    pub const LANGUID_LAVENDER: Self = Self::from_rgb8(0xd6, 0xca, 0xdd);
    pub const LAPIS_LAZULI: Self = Self::from_rgb8(0x26, 0x61, 0x9c);
    pub const LASER_LEMON: Self = Self::from_rgb8(0xff, 0xff, 0x66);
    pub const LAUREL_GREEN: Self = Self::from_rgb8(0xa9, 0xba, 0x9d);
    pub const LAVENDER: Self = Self::from_rgb8(0xe6, 0xe6, 0xfa);
    pub const LAVENDER_FLORAL: Self = Self::from_rgb8(0xb5, 0x7e, 0xdc);
    pub const LAVENDER_BLUE: Self = Self::from_rgb8(0xcc, 0xcc, 0xff);
    pub const LAVENDER_GRAY: Self = Self::from_rgb8(0xc4, 0xc3, 0xd0);
    pub const LAVENDER_BLUSH: Self = Self::from_rgb8(0xff, 0xf0, 0xf5);
    pub const LAWNGREEN: Self = Self::from_rgb8(0x7c, 0xfc, 0x00);
    pub const LEMON: Self = Self::from_rgb8(0xff, 0xf7, 0x00);
    pub const LEMON_CURRY: Self = Self::from_rgb8(0xcc, 0xa0, 0x1d);
    pub const LEMON_GLACIER: Self = Self::from_rgb8(0xfd, 0xff, 0x00);
    pub const LEMON_MERINGUE: Self = Self::from_rgb8(0xf6, 0xea, 0xbe);
    pub const LEMON_YELLOW: Self = Self::from_rgb8(0xff, 0xf4, 0x4f);
    pub const LEMON_CHIFFON: Self = Self::from_rgb8(0xff, 0xfa, 0xcd);
    pub const LIGHT: Self = Self::from_rgb8(0xee, 0xdd, 0x82);
    pub const LIGHT_CORNFLOWER_BLUE: Self = Self::from_rgb8(0x93, 0xcc, 0xea);
    pub const LIGHT_FRENCH_BEIGE: Self = Self::from_rgb8(0xc8, 0xad, 0x7f);
    pub const LIGHT_ORANGE: Self = Self::from_rgb8(0xfe, 0xd8, 0xb1);
    pub const LIGHT_PERIWINKLE: Self = Self::from_rgb8(0xc5, 0xcb, 0xe1);
    pub const LIGHT_BLUE: Self = Self::from_rgb8(0xad, 0xd8, 0xe6);
    pub const LIGHT_CORAL: Self = Self::from_rgb8(0xf0, 0x80, 0x80);
    pub const LIGHT_CYAN: Self = Self::from_rgb8(0xe0, 0xff, 0xff);
    pub const LIGHT_GOLDEN_ROD: Self = Self::from_rgb8(0xff, 0xec, 0x8b);
    pub const LIGHT_GOLDEN_ROD_YELLOW: Self = Self::from_rgb8(0xfa, 0xfa, 0xd2);
    pub const LIGHT_GRAY: Self = Self::from_rgb8(0xd3, 0xd3, 0xd3);
    pub const LIGHT_PINK: Self = Self::from_rgb8(0xff, 0xb6, 0xc1);
    pub const LIGHT_SALMON: Self = Self::from_rgb8(0xff, 0xa0, 0x7a);
    pub const LIGHT_SEA_GREEN: Self = Self::from_rgb8(0x20, 0xb2, 0xaa);
    pub const LIGHT_SKY_BLUE: Self = Self::from_rgb8(0x87, 0xce, 0xfa);
    pub const LIGHT_SLATE_BLUE: Self = Self::from_rgb8(0x84, 0x70, 0xff);
    pub const LIGHT_SLATE_GRAY: Self = Self::from_rgb8(0x77, 0x88, 0x99);
    pub const LIGHT_STEEL_BLUE: Self = Self::from_rgb8(0xb0, 0xc4, 0xde);
    pub const LIGHT_YELLOW: Self = Self::from_rgb8(0xff, 0xff, 0xe0);
    pub const LILAC: Self = Self::from_rgb8(0xc8, 0xa2, 0xc8);
    pub const LILAC_LUSTER: Self = Self::from_rgb8(0xae, 0x98, 0xaa);
    pub const LIMEGREEN: Self = Self::from_rgb8(0x32, 0xcd, 0x32);
    pub const LINCOLN_GREEN: Self = Self::from_rgb8(0x19, 0x59, 0x05);
    pub const LINEN: Self = Self::from_rgb8(0xfa, 0xf0, 0xe6);
    pub const LITTLE_BOY_BLUE: Self = Self::from_rgb8(0x6c, 0xa0, 0xdc);
    pub const MSU_GREEN: Self = Self::from_rgb8(0x18, 0x45, 0x3b);
    pub const MACARONI_AND_CHEESE: Self = Self::from_rgb8(0xff, 0xbd, 0x88);
    pub const MADDER_LAKE: Self = Self::from_rgb8(0xcc, 0x33, 0x36);
    pub const MAGENTA: Self = Self::from_rgb8(0xff, 0x00, 0xff);
    pub const MAGENTA_CRAYOLA: Self = Self::from_rgb8(0xf6, 0x53, 0xa6);
    pub const MAGENTA_PANTONE: Self = Self::from_rgb8(0xd0, 0x41, 0x7e);
    pub const MAGENTA_HAZE: Self = Self::from_rgb8(0x9f, 0x45, 0x76);
    pub const MAGIC_MINT: Self = Self::from_rgb8(0xaa, 0xf0, 0xd1);
    pub const MAHOGANY: Self = Self::from_rgb8(0xc0, 0x40, 0x00);
    pub const MAJORELLE_BLUE: Self = Self::from_rgb8(0x60, 0x50, 0xdc);
    pub const MALACHITE: Self = Self::from_rgb8(0x0b, 0xda, 0x51);
    pub const MANATEE: Self = Self::from_rgb8(0x97, 0x9a, 0xaa);
    pub const MANDARIN: Self = Self::from_rgb8(0xf3, 0x7a, 0x48);
    pub const MANGO: Self = Self::from_rgb8(0xfd, 0xbe, 0x02);
    pub const MANGO_TANGO: Self = Self::from_rgb8(0xff, 0x82, 0x43);
    pub const MANTIS: Self = Self::from_rgb8(0x74, 0xc3, 0x65);
    pub const MARIGOLD: Self = Self::from_rgb8(0xea, 0xa2, 0x21);
    pub const MAROON: Self = Self::from_rgb8(0xb0, 0x30, 0x60);
    pub const MAUVE: Self = Self::from_rgb8(0xe0, 0xb0, 0xff);
    pub const MAUVE_TAUPE: Self = Self::from_rgb8(0x91, 0x5f, 0x6d);
    pub const MAUVELOUS: Self = Self::from_rgb8(0xef, 0x98, 0xaa);
    pub const MAXIMUM_BLUE_GREEN: Self = Self::from_rgb8(0x30, 0xbf, 0xbf);
    pub const MAXIMUM_BLUE_PURPLE: Self = Self::from_rgb8(0xac, 0xac, 0xe6);
    pub const MAXIMUM_GREEN: Self = Self::from_rgb8(0x5e, 0x8c, 0x31);
    pub const MAXIMUM_BLUE: Self = Self::from_rgb8(0x47, 0xab, 0xcc);
    pub const MAY_GREEN: Self = Self::from_rgb8(0x4c, 0x91, 0x41);
    pub const MAYA_BLUE: Self = Self::from_rgb8(0x73, 0xc2, 0xfb);
    pub const MEDIUM: Self = Self::from_rgb8(0x66, 0xcd, 0xaa);
    pub const MEDIUM_AQUAMARINE: Self = Self::from_rgb8(0x66, 0xdd, 0xaa);
    pub const MEDIUM_CANDY_APPLE_RED: Self = Self::from_rgb8(0xe2, 0x06, 0x2c);
    pub const MEDIUM_CARMINE: Self = Self::from_rgb8(0xaf, 0x40, 0x35);
    pub const MEDIUM_CHAMPAGNE: Self = Self::from_rgb8(0xf3, 0xe5, 0xab);
    pub const MEDIUMAQUAMARINE: Self = Self::from_rgb8(0x66, 0xcd, 0xaa);
    pub const MEDIUMBLUE: Self = Self::from_rgb8(0x00, 0x00, 0xcd);
    pub const MEDIUMORCHID: Self = Self::from_rgb8(0xba, 0x55, 0xd3);
    pub const MEDIUMPURPLE: Self = Self::from_rgb8(0x93, 0x70, 0xdb);
    pub const MEDIUMSEAGREEN: Self = Self::from_rgb8(0x3c, 0xb3, 0x71);
    pub const MEDIUMSLATEBLUE: Self = Self::from_rgb8(0x7b, 0x68, 0xee);
    pub const MEDIUMSPRINGGREEN: Self = Self::from_rgb8(0x00, 0xfa, 0x9a);
    pub const MEDIUMTURQUOISE: Self = Self::from_rgb8(0x48, 0xd1, 0xcc);
    pub const MEDIUMVIOLETRED: Self = Self::from_rgb8(0xc7, 0x15, 0x85);
    pub const MELLOW_APRICOT: Self = Self::from_rgb8(0xf8, 0xb8, 0x78);
    pub const MELON: Self = Self::from_rgb8(0xfe, 0xba, 0xad);
    pub const METALLIC_GOLD: Self = Self::from_rgb8(0xd3, 0xaf, 0x37);
    pub const METALLIC_SEAWEED: Self = Self::from_rgb8(0x0a, 0x7e, 0x8c);
    pub const METALLIC_SUNBURST: Self = Self::from_rgb8(0x9c, 0x7c, 0x38);
    pub const MEXICAN_PINK: Self = Self::from_rgb8(0xe4, 0x00, 0x7c);
    pub const MIDDLE_BLUE: Self = Self::from_rgb8(0x7e, 0xd4, 0xe6);
    pub const MIDDLE_BLUE_GREEN: Self = Self::from_rgb8(0x8d, 0xd9, 0xcc);
    pub const MIDDLE_BLUE_PURPLE: Self = Self::from_rgb8(0x8b, 0x72, 0xbe);
    pub const MIDDLE_GREEN: Self = Self::from_rgb8(0x4d, 0x8c, 0x57);
    pub const MIDDLE_GREEN_YELLOW: Self = Self::from_rgb8(0xac, 0xbf, 0x60);
    pub const MIDDLE_GREY: Self = Self::from_rgb8(0x8b, 0x86, 0x80);
    pub const MIDDLE_PURPLE: Self = Self::from_rgb8(0xd9, 0x82, 0xb5);
    pub const MIDDLE_RED: Self = Self::from_rgb8(0xe5, 0x8e, 0x73);
    pub const MIDDLE_RED_PURPLE: Self = Self::from_rgb8(0xa5, 0x53, 0x53);
    pub const MIDDLE_YELLOW: Self = Self::from_rgb8(0xff, 0xeb, 0x00);
    pub const MIDDLE_YELLOW_RED: Self = Self::from_rgb8(0xec, 0xb1, 0x76);
    pub const MIDNIGHT_GREEN: Self = Self::from_rgb8(0x00, 0x49, 0x53);
    pub const MIDNIGHTBLUE: Self = Self::from_rgb8(0x19, 0x19, 0x70);
    pub const MIKADO_YELLOW: Self = Self::from_rgb8(0xff, 0xc4, 0x0c);
    pub const MIMI_PINK: Self = Self::from_rgb8(0xff, 0xda, 0xe9);
    pub const MINDARO: Self = Self::from_rgb8(0xe3, 0xf9, 0x88);
    pub const MINION_YELLOW: Self = Self::from_rgb8(0xf5, 0xe0, 0x50);
    pub const MINT: Self = Self::from_rgb8(0x3e, 0xb4, 0x89);
    pub const MINT_GREEN: Self = Self::from_rgb8(0x98, 0xff, 0x98);
    pub const MINTCREAM: Self = Self::from_rgb8(0xf5, 0xff, 0xfa);
    pub const MISTY_MOSS: Self = Self::from_rgb8(0xbb, 0xb4, 0x77);
    pub const MISTY_ROSE: Self = Self::from_rgb8(0xff, 0xe4, 0xe1);
    pub const MOCCASIN: Self = Self::from_rgb8(0xff, 0xe4, 0xb5);
    pub const MODE_BEIGE: Self = Self::from_rgb8(0x96, 0x71, 0x17);
    pub const MOSS_GREEN: Self = Self::from_rgb8(0x8a, 0x9a, 0x5b);
    pub const MOUNTAIN_MEADOW: Self = Self::from_rgb8(0x30, 0xba, 0x8f);
    pub const MOUNTBATTEN_PINK: Self = Self::from_rgb8(0x99, 0x7a, 0x8d);
    pub const MULBERRY: Self = Self::from_rgb8(0xc5, 0x4b, 0x8c);
    pub const MUSTARD: Self = Self::from_rgb8(0xff, 0xdb, 0x58);
    pub const MYRTLE_GREEN: Self = Self::from_rgb8(0x31, 0x78, 0x73);
    pub const MYSTIC_MAROON: Self = Self::from_rgb8(0xad, 0x43, 0x79);
    pub const NADESHIKO_PINK: Self = Self::from_rgb8(0xf6, 0xad, 0xc6);
    pub const NAVAJO_WHITE: Self = Self::from_rgb8(0xff, 0xde, 0xad);
    pub const NAVYBLUE: Self = Self::from_rgb8(0x00, 0x00, 0x80);
    pub const NEON_BLUE: Self = Self::from_rgb8(0x46, 0x66, 0xff);
    pub const NEON_CARROT: Self = Self::from_rgb8(0xff, 0xa3, 0x43);
    pub const NEON_FUCHSIA: Self = Self::from_rgb8(0xfe, 0x41, 0x64);
    pub const NEON_GREEN: Self = Self::from_rgb8(0x39, 0xff, 0x14);
    pub const NICKEL: Self = Self::from_rgb8(0x72, 0x74, 0x72);
    pub const NYANZA: Self = Self::from_rgb8(0xe9, 0xff, 0xdb);
    pub const OCEAN_BLUE: Self = Self::from_rgb8(0x4f, 0x42, 0xb5);
    pub const OCEAN_GREEN: Self = Self::from_rgb8(0x48, 0xbf, 0x91);
    pub const OCHRE: Self = Self::from_rgb8(0xcc, 0x77, 0x22);
    pub const OFF_WHITE: Self = Self::from_rgb8(0xf2, 0xf0, 0xef);
    pub const OLD_BURGUNDY: Self = Self::from_rgb8(0x43, 0x30, 0x2e);
    pub const OLD_GOLD: Self = Self::from_rgb8(0xcf, 0xb5, 0x3b);
    pub const OLD_LAVENDER: Self = Self::from_rgb8(0x79, 0x68, 0x78);
    pub const OLD_MAUVE: Self = Self::from_rgb8(0x67, 0x31, 0x47);
    pub const OLD_ROSE: Self = Self::from_rgb8(0xc0, 0x80, 0x81);
    pub const OLDLACE: Self = Self::from_rgb8(0xfd, 0xf5, 0xe6);
    pub const OLIVE: Self = Self::from_rgb8(0x80, 0x80, 0x00);
    pub const OLIVE_GREEN: Self = Self::from_rgb8(0xb5, 0xb3, 0x5c);
    pub const OLIVEDRAB: Self = Self::from_rgb8(0x6b, 0x8e, 0x23);
    pub const OLIVINE: Self = Self::from_rgb8(0x9a, 0xb9, 0x73);
    pub const OPAL: Self = Self::from_rgb8(0xa8, 0xc3, 0xbc);
    pub const OPERA_MAUE: Self = Self::from_rgb8(0xb7, 0x84, 0xa7);
    pub const ORANGE: Self = Self::from_rgb8(0xff, 0x58, 0x00);
    pub const ORANGE_PEEL: Self = Self::from_rgb8(0xff, 0x9f, 0x00);
    pub const ORANGE_SODA: Self = Self::from_rgb8(0xfa, 0x5b, 0x3d);
    pub const ORANGE_RED: Self = Self::from_rgb8(0xff, 0x45, 0x00);
    pub const ORCHID: Self = Self::from_rgb8(0xda, 0x70, 0xd6);
    pub const ORCHID_PINK: Self = Self::from_rgb8(0xf2, 0xbd, 0xcd);
    pub const OUTRAGEOUS_ORANGE: Self = Self::from_rgb8(0xff, 0x6e, 0x4a);
    pub const OXBLOOD: Self = Self::from_rgb8(0x4a, 0x00, 0x00);
    pub const OXFORD_BLUE: Self = Self::from_rgb8(0x00, 0x21, 0x47);
    pub const PACIFIC_BLUE: Self = Self::from_rgb8(0x1c, 0xa9, 0xc9);
    pub const PALATINATE_PURPLE: Self = Self::from_rgb8(0x68, 0x28, 0x60);
    pub const PALE: Self = Self::from_rgb8(0xdb, 0x70, 0x93);
    pub const PALE_AQUA: Self = Self::from_rgb8(0xbc, 0xd4, 0xe6);
    pub const PALE_CERULEAN: Self = Self::from_rgb8(0x9b, 0xc4, 0xe2);
    pub const PALE_PINK: Self = Self::from_rgb8(0xfa, 0xda, 0xdd);
    pub const PALE_SILVER: Self = Self::from_rgb8(0xc9, 0xc0, 0xbb);
    pub const PALE_SPRING_BUD: Self = Self::from_rgb8(0xec, 0xeb, 0xbd);
    pub const PALEGOLDENROD: Self = Self::from_rgb8(0xee, 0xe8, 0xaa);
    pub const PALEGREEN: Self = Self::from_rgb8(0x98, 0xfb, 0x98);
    pub const PALETURQUOISE: Self = Self::from_rgb8(0xaf, 0xee, 0xee);
    pub const PALEVIOLETRED: Self = Self::from_rgb8(0xdb, 0x70, 0x93);
    pub const PANSY_PURPLE: Self = Self::from_rgb8(0x78, 0x18, 0x4a);
    pub const PAPAYAWHIP: Self = Self::from_rgb8(0xff, 0xef, 0xd5);
    pub const PARADISE_PINK: Self = Self::from_rgb8(0xe6, 0x3e, 0x62);
    pub const PASTEL_PINK: Self = Self::from_rgb8(0xde, 0xa5, 0xa4);
    pub const PATRIARCH_PURPLE: Self = Self::from_rgb8(0x80, 00, 80);
    pub const PEACH: Self = Self::from_rgb8(0xff, 0xe5, 0xb4);
    pub const PEACHPUFF: Self = Self::from_rgb8(0xff, 0xda, 0xb9);
    pub const PEAR: Self = Self::from_rgb8(0xd1, 0xe2, 0x31);
    pub const PEARLY_PURPLE: Self = Self::from_rgb8(0xb7, 0x68, 0xa2);
    pub const PERSIAN_BLUE: Self = Self::from_rgb8(0x1c, 0x39, 0xbb);
    pub const PERSIAN_GREEN: Self = Self::from_rgb8(0x00, 0xa6, 0x93);
    pub const PERSIAN_INDIGO: Self = Self::from_rgb8(0x32, 0x12, 0x7a);
    pub const PERSIAN_ORANGE: Self = Self::from_rgb8(0xd9, 0x90, 0x58);
    pub const PERSIAN_PINK: Self = Self::from_rgb8(0xf7, 0x7f, 0xbe);
    pub const PERSIAN_PLUM: Self = Self::from_rgb8(0x70, 0x1c, 0x1c);
    pub const PERSIAN_RED: Self = Self::from_rgb8(0xcc, 0x33, 0x33);
    pub const PERSIAN_ROSE: Self = Self::from_rgb8(0xfe, 0x28, 0xa2);
    pub const PEWTER_BLUE: Self = Self::from_rgb8(0x8b, 0xa8, 0xb7);
    pub const PHTHALO_BLUE: Self = Self::from_rgb8(0x00, 0x0f, 0x89);
    pub const PHTHALO_GREEN: Self = Self::from_rgb8(0x12, 0x35, 0x24);
    pub const PICTORIAL_CARMINE: Self = Self::from_rgb8(0xc3, 0x0b, 0x4e);
    pub const PIGGY_PINK: Self = Self::from_rgb8(0xfd, 0xdd, 0xe6);
    pub const PINE_GREEN: Self = Self::from_rgb8(0x01, 0x79, 0x6f);
    pub const PINE_TREE: Self = Self::from_rgb8(0x2a, 0x2f, 0x23);
    pub const PINK: Self = Self::from_rgb8(0xff, 0xc0, 0xcb);
    pub const PINK_PANTONE: Self = Self::from_rgb8(0xd7, 0x48, 0x94);
    pub const PINK_FLAMINGO: Self = Self::from_rgb8(0xfc, 0x74, 0xfd);
    pub const PINK_SHERBET: Self = Self::from_rgb8(0xf7, 0x8f, 0xa7);
    pub const PISTACHIO: Self = Self::from_rgb8(0x93, 0xc5, 0x72);
    pub const PLATINUM: Self = Self::from_rgb8(0xe5, 0xe4, 0xe2);
    pub const PLUM: Self = Self::from_rgb8(0x8e, 0x45, 0x85);
    pub const PLUMP_PURPLE: Self = Self::from_rgb8(0x59, 0x46, 0xb2);
    pub const PORTLAND_ORANGE: Self = Self::from_rgb8(0xff, 0x5a, 0x36);
    pub const POWDERBLUE: Self = Self::from_rgb8(0xb0, 0xe0, 0xe6);
    pub const PRUSSIAN_BLUE: Self = Self::from_rgb8(0x00, 0x31, 0x53);
    pub const PUCE: Self = Self::from_rgb8(0xcc, 0x88, 0x99);
    pub const PUMPKIN: Self = Self::from_rgb8(0xff, 0x75, 0x18);
    pub const PURPLE: Self = Self::from_rgb8(0xa0, 0x20, 0xf0);
    pub const QUINACRIDONE_MAGENTA: Self = Self::from_rgb8(0x8e, 0x3a, 0x59);
    pub const RADICAL_RED: Self = Self::from_rgb8(0xff, 0x35, 0x5e);
    pub const RASPBERRY: Self = Self::from_rgb8(0xe3, 0x0b, 0x5d);
    pub const RAZZMATAZZ: Self = Self::from_rgb8(0xe3, 0x25, 0x6b);
    pub const REBECCAPURPLE: Self = Self::from_rgb8(0x66, 0x33, 0x99);
    pub const RED_ORANGE: Self = Self::from_rgb8(0xff, 0x53, 0x49);
    pub const RED: Self = Self::from_rgb8(0xff, 0x00, 0x00);
    pub const REDWOOD: Self = Self::from_rgb8(0xa4, 0x5a, 0x52);
    pub const RIFLE_GREEN: Self = Self::from_rgb8(0x44, 0x4c, 0x38);
    pub const ROCKET_METALLIC: Self = Self::from_rgb8(0x8a, 0x7f, 0x80);
    pub const ROSE: Self = Self::from_rgb8(0xff, 0x00, 0x7f);
    pub const ROSE_BONBON: Self = Self::from_rgb8(0xf9, 0x42, 0x9e);
    pub const ROSE_DUST: Self = Self::from_rgb8(0x9e, 0x5e, 0x6f);
    pub const ROSE_PINK: Self = Self::from_rgb8(0xff, 0x66, 0xcc);
    pub const ROSE_TAUPE: Self = Self::from_rgb8(0x90, 0x5d, 0x5d);
    pub const ROSEWOOD: Self = Self::from_rgb8(0x65, 0x00, 0x0b);
    pub const ROSYBROWN: Self = Self::from_rgb8(0xbc, 0x8f, 0x8f);
    pub const ROYALBLUE: Self = Self::from_rgb8(0x41, 0x69, 0xe1);
    pub const RUBY: Self = Self::from_rgb8(0xe0, 0x11, 0x5f);
    pub const RUSSET: Self = Self::from_rgb8(0x80, 0x46, 0x1b);
    pub const RUSSIAN_GREEN: Self = Self::from_rgb8(0x67, 0x92, 0x67);
    pub const RUSSIAN_VIOLET: Self = Self::from_rgb8(0x32, 0x17, 0x4d);
    pub const RUST: Self = Self::from_rgb8(0xb7, 0x41, 0x0e);
    pub const SADDLEBROWN: Self = Self::from_rgb8(0x8b, 0x45, 0x13);
    pub const SAFFRON: Self = Self::from_rgb8(0xf4, 0xc4, 0x30);
    pub const SAGE: Self = Self::from_rgb8(0xbc, 0xb8, 0x8a);
    pub const SALMON: Self = Self::from_rgb8(0xfa, 0x80, 0x72);
    pub const SANDYBROWN: Self = Self::from_rgb8(0xf4, 0xa4, 0x60);
    pub const SAP_GREEN: Self = Self::from_rgb8(0x50, 0x7d, 0x2a);
    pub const SAPPHIRE: Self = Self::from_rgb8(0x0f, 0x52, 0xba);
    pub const SCARLET: Self = Self::from_rgb8(0xff, 0x24, 0x00);
    pub const SCHOOL_BUS_YELLOW: Self = Self::from_rgb8(0xff, 0xd8, 0x00);
    pub const SEAGREEN: Self = Self::from_rgb8(0x54, 0xff, 0x9f);
    pub const SEAL_BROWN: Self = Self::from_rgb8(0x59, 0x26, 0x0b);
    pub const SEASHELL: Self = Self::from_rgb8(0xff, 0xf5, 0xee);
    pub const SELECTIVE_YELLOW: Self = Self::from_rgb8(0xff, 0xba, 0x00);
    pub const SEPIA: Self = Self::from_rgb8(0x70, 0x42, 0x14);
    pub const SHAMROCK_GREEN: Self = Self::from_rgb8(0x00, 0x9e, 0x60);
    pub const SHOCKING_PINK: Self = Self::from_rgb8(0xfc, 0x0f, 0xc0);
    pub const SIENNA: Self = Self::from_rgb8(0xa0, 0x52, 0x2d);
    pub const SILVER: Self = Self::from_rgb8(0xc0, 0xc0, 0xc0);
    pub const SILVER_PINK: Self = Self::from_rgb8(0xc4, 0xae, 0xad);
    pub const SINOPIA: Self = Self::from_rgb8(0xcb, 0x41, 0x0b);
    pub const SKOBELOFF: Self = Self::from_rgb8(0x00, 0x74, 0x74);
    pub const SKYBLUE: Self = Self::from_rgb8(0x87, 0xce, 0xeb);
    pub const SLATEBLUE: Self = Self::from_rgb8(0x6a, 0x5a, 0xcd);
    pub const SLATEGRAY: Self = Self::from_rgb8(0x70, 0x80, 0x90);
    pub const SMOKY_BLACK: Self = Self::from_rgb8(0x10, 0x0c, 0x08);
    pub const SNOW: Self = Self::from_rgb8(0xff, 0xfa, 0xfa);
    pub const SPANISH_BISTRE: Self = Self::from_rgb8(0x80, 0x75, 0x32);
    pub const SPANISH_ORANGE: Self = Self::from_rgb8(0xe8, 0x61, 0x00);
    pub const SPANISH_PINK: Self = Self::from_rgb8(0xf7, 0xbf, 0xbe);
    pub const SPANISH_VIRIDIAN: Self = Self::from_rgb8(0x00, 0x7f, 0x5c);
    pub const SPRING_BUD: Self = Self::from_rgb8(0xa7, 0xfc, 0x00);
    pub const SPRING_FROST: Self = Self::from_rgb8(0x87, 0xff, 0x2a);
    pub const SPRINGGREEN: Self = Self::from_rgb8(0x00, 0xff, 0x7f);
    pub const STEEL_PINK: Self = Self::from_rgb8(0xcc, 0x33, 0xcc);
    pub const STEELBLUE: Self = Self::from_rgb8(0x46, 0x82, 0xb4);
    pub const STRAW: Self = Self::from_rgb8(0xe4, 0xd9, 0x6f);
    pub const SUNGLOW: Self = Self::from_rgb8(0xff, 0xcc, 0x33);
    pub const SUPER_PINK: Self = Self::from_rgb8(0xcf, 0x6b, 0xa9);
    pub const SWEET_BROWN: Self = Self::from_rgb8(0xa8, 0x37, 0x31);
    pub const TAN: Self = Self::from_rgb8(0xd2, 0xb4, 0x8c);
    pub const TANGERINE: Self = Self::from_rgb8(0xf2, 0x85, 0x00);
    pub const TART_ORANGE: Self = Self::from_rgb8(0xfb, 0x4d, 0x46);
    pub const TAUPE: Self = Self::from_rgb8(0x48, 0x3c, 0x32);
    pub const TAUPE_GRAY: Self = Self::from_rgb8(0x8b, 0x85, 0x89);
    pub const TEA_GREEN: Self = Self::from_rgb8(0xd0, 0xf0, 0xc0);
    pub const TEAL: Self = Self::from_rgb8(0x00, 0x80, 0x80);
    pub const TEAL_BLUE: Self = Self::from_rgb8(0x36, 0x75, 0x88);
    pub const TERRA_COTTA: Self = Self::from_rgb8(0xe2, 0x72, 0x5b);
    pub const THISTLE: Self = Self::from_rgb8(0xd8, 0xbf, 0xd8);
    pub const TIFFANY_BLUE: Self = Self::from_rgb8(0x0a, 0xba, 0xb5);
    pub const TIMBERWOLF: Self = Self::from_rgb8(0xdb, 0xd7, 0xd2);
    pub const TITANIUM_YELLOW: Self = Self::from_rgb8(0xee, 0xe6, 0x00);
    pub const TOMATO: Self = Self::from_rgb8(0xff, 0x63, 0x47);
    pub const TROPICAL_RAINFOREST: Self = Self::from_rgb8(0x00, 0x75, 0x5e);
    pub const TUMBLEWEED: Self = Self::from_rgb8(0xde, 0xaa, 0x88);
    pub const TURQUOISE: Self = Self::from_rgb8(0x40, 0xe0, 0xd0);
    pub const TURQUOISE_BLUE: Self = Self::from_rgb8(0x00, 0xff, 0xef);
    pub const TUSCAN_RED: Self = Self::from_rgb8(0x7c, 0x48, 0x48);
    pub const TUSCANY: Self = Self::from_rgb8(0xc0, 0x99, 0x99);
    pub const TWILIGHT_LAVENDER: Self = Self::from_rgb8(0x8a, 0x49, 0x6b);
    pub const TYRIAN_PURPLE: Self = Self::from_rgb8(0x66, 0x02, 0x3c);
    pub const UP_FOREST_GREEN: Self = Self::from_rgb8(0x01, 0x44, 0x21);
    pub const UP_MAROON: Self = Self::from_rgb8(0x7b, 0x11, 0x13);
    pub const ULTRA_PINK: Self = Self::from_rgb8(0xff, 0x6f, 0xff);
    pub const ULTRAMARINE: Self = Self::from_rgb8(0x3f, 0x00, 0xff);
    pub const ULTRAMARINE_BLUE: Self = Self::from_rgb8(0x41, 0x66, 0xf5);
    pub const UNBLEACHED_SILK: Self = Self::from_rgb8(0xff, 0xdd, 0xca);
    pub const UNITED_NATIONS_BLUE: Self = Self::from_rgb8(0x5b, 0x92, 0xe5);
    pub const UPSDELL_RED: Self = Self::from_rgb8(0xae, 0x20, 0x29);
    pub const VAN_DYKE_BROWN: Self = Self::from_rgb8(0x66, 0x42, 0x28);
    pub const VANILLA: Self = Self::from_rgb8(0xf3, 0xe5, 0xab);
    pub const VANILLA_ICE: Self = Self::from_rgb8(0xf3, 0x8f, 0xa9);
    pub const VEGAS_GOLD: Self = Self::from_rgb8(0xc5, 0xb3, 0x58);
    pub const VENETIAN_RED: Self = Self::from_rgb8(0xc8, 0x08, 0x15);
    pub const VERDIGRIS: Self = Self::from_rgb8(0x43, 0xb3, 0xae);
    pub const VERMILLION: Self = Self::from_rgb8(0xe3, 0x42, 0x34);
    pub const VIOLET: Self = Self::from_rgb8(0xee, 0x82, 0xee);
    pub const VIOLETRED: Self = Self::from_rgb8(0xd0, 0x20, 0x90);
    pub const VIRIDIAN: Self = Self::from_rgb8(0x40, 0x82, 0x6d);
    pub const VIRIDIAN_GREEN: Self = Self::from_rgb8(0x00, 0x96, 0x98);
    pub const VIVID_BURGUNDY: Self = Self::from_rgb8(0x9f, 0x1d, 0x35);
    pub const VIVID_SKY_BLUE: Self = Self::from_rgb8(0x00, 0xcc, 0xff);
    pub const VIVID_TANGERINE: Self = Self::from_rgb8(0xff, 0xa0, 0x89);
    pub const VIVID_VIOLET: Self = Self::from_rgb8(0x9f, 0x00, 0xff);
    pub const VOLT: Self = Self::from_rgb8(0xce, 0xff, 0x00);
    pub const WARM_BLACK: Self = Self::from_rgb8(0x00, 0x42, 0x42);
    pub const WHEAT: Self = Self::from_rgb8(0xf5, 0xde, 0xb3);
    pub const WHITE: Self = Self::from_rgb8(0xff, 0xff, 0xff);
    pub const WHITESMOKE: Self = Self::from_rgb8(0xf5, 0xf5, 0xf5);
    pub const WILD_BLUE_YONDER: Self = Self::from_rgb8(0xa2, 0xad, 0xd0);
    pub const WILD_ORCHID: Self = Self::from_rgb8(0xd4, 0x70, 0xa2);
    pub const WILD_STRAWBERRY: Self = Self::from_rgb8(0xff, 0x43, 0xa4);
    pub const WINDSOR_TAN: Self = Self::from_rgb8(0xa7, 0x55, 0x02);
    pub const WINE: Self = Self::from_rgb8(0x72, 0x2f, 0x37);
    pub const WINTERGREEN_DREAM: Self = Self::from_rgb8(0x56, 0x88, 0x7d);
    pub const WISTERIA: Self = Self::from_rgb8(0xc9, 0xa0, 0xdc);
    pub const XANADU: Self = Self::from_rgb8(0x73, 0x86, 0x78);
    pub const YELLOW_ORANGE: Self = Self::from_rgb8(0xff, 0xae, 0x42);
    pub const YELLOW_PANTONE: Self = Self::from_rgb8(0xfe, 0xdf, 0x00);
    pub const YELLOW: Self = Self::from_rgb8(0xff, 0xff, 0x00);
    pub const YELLOWGREEN: Self = Self::from_rgb8(0x9a, 0xcd, 0x32);
    pub const ZAFFRE: Self = Self::from_rgb8(0x00, 0x14, 0xa8);
    pub const ZOMP: Self = Self::from_rgb8(0x39, 0xa7, 0x8e);
}
