enum EnvelopeDirection {
    Increase,
    Decrease,
}

impl From<u8> for EnvelopeDirection {
    fn from(val: u8) -> Self {
        match val {
            0 => Self::Decrease,
            1 => Self::Increase,
            _ => unreachable!("Invalid envelope direction value"),
        }
    }
}

struct Envelope {
    initial_vol: u8,
    direction: EnvelopeDirection,
    period: u8,

    period_timer: u8,
    current_vol: u8,
}

impl Default for Envelope {
    fn default() -> Self {
        Self {
            direction: EnvelopeDirection::Decrease,
            ..Default::default()
        }
    }
}

impl Envelope {
    pub fn init(&mut self, nrx2: u8) -> Self {
        let period = nrx2 & 0b0000_0111;
        let initial_vol = (nrx2 & 0b1111_0000) >> 4;

        Self {
            initial_vol,
            direction: ((nrx2 & 0b0000_1000) >> 3).into(),
            period,

            period_timer: period,
            current_vol: initial_vol,
        }
    }

    fn is_current_vol_not_at_boundary(&self) -> bool {
        match self.direction {
            EnvelopeDirection::Increase => self.current_vol < 0xF,
            EnvelopeDirection::Decrease => self.current_vol > 0x0,
        }
    }

    fn step_volume(&mut self) {
        match self.direction {
            EnvelopeDirection::Increase => self.current_vol += 1,
            EnvelopeDirection::Decrease => self.current_vol -= 1,
        }
    }

    fn current_vol(&self) -> u8 {
        self.current_vol
    }

    pub fn tick(&mut self) {
        if self.period == 0 {
            return;
        }

        if self.period_timer > 0 {
            self.period_timer -= 1;
        }

        if self.period_timer == 0 {
            self.period_timer = self.period;

            if self.is_current_vol_not_at_boundary() {
                self.step_volume();
            }
        }
    }
}
