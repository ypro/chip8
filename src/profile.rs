pub struct Profile {
    pub op_8xy6_use_vy: bool,
    pub op_8xye_use_vy: bool,
    pub op_fx55_store_i: bool,
    pub op_fx65_store_i: bool,
}

impl Profile {
    pub fn original() -> Profile {
        Profile {
            op_8xy6_use_vy: true,
            op_8xye_use_vy: true,
            op_fx55_store_i: true,
            op_fx65_store_i: true,
        }
    }

    pub fn modern() -> Profile {
        Profile {
            op_8xy6_use_vy: false,
            op_8xye_use_vy: false,
            op_fx55_store_i: false,
            op_fx65_store_i: false,
        }
    }
}
