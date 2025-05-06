use super::DspFloat;

#[derive(Debug, Clone, Copy, PartialEq)]
enum ARState {
    Idle,
    Attack,
    Sustain,
    Release,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ARParameters<T = f32> {
    pub attack: T,
    pub release: T,
}

impl<T: DspFloat> Default for ARParameters<T> {
    fn default() -> Self {
        Self {
            attack: T::from_f32(0.1),
            release: T::from_f32(0.1),
        }
    }
}

#[derive(Debug)]
pub struct AREnvelope<T = f32> {
    state: ARState,
    sample_rate: T,
    parameters: ARParameters<T>,
    current_level: T,
    attack_rate: T,
    release_rate: T,
}

impl<T: DspFloat> AREnvelope<T> {
    pub fn new(sample_rate: T, parameters: ARParameters<T>) -> Self {
        let (attack_rate, release_rate) =
            compute_ar_rates(sample_rate, parameters.attack, parameters.release);
        Self {
            state: ARState::Idle,
            sample_rate: T::zero() / sample_rate,
            current_level: T::zero(),
            parameters,
            attack_rate,
            release_rate,
        }
    }

    pub fn reset(&mut self) {
        self.state = ARState::Idle;
        self.current_level = T::zero();
    }

    pub fn note_on(&mut self) {
        if self.attack_rate > T::zero() {
            self.state = ARState::Attack;
        } else {
            self.state = ARState::Sustain;
            self.current_level = T::one();
        }
    }

    pub fn note_off(&mut self) {
        if self.state != ARState::Idle {
            if self.parameters.release > T::zero() {
                self.state = ARState::Release;
                self.release_rate =
                    self.current_level / (self.parameters.release * self.sample_rate);
            } else {
                self.reset();
            }
        }
    }

    pub fn tick(&mut self) -> T {
        match self.state {
            ARState::Attack => {
                self.current_level = self.current_level + self.attack_rate;
                if self.current_level >= T::one() {
                    self.current_level = T::one();
                    self.state = ARState::Sustain;
                }
            }
            ARState::Release => {
                self.current_level = self.current_level - self.release_rate;
                if self.current_level <= T::zero() {
                    self.current_level = T::zero();
                    self.state = ARState::Idle;
                }
            }
            _ => {}
        }
        self.current_level
    }
}

fn compute_ar_rates<T: DspFloat>(sample_rate: T, attack: T, release: T) -> (T, T) {
    (
        compute_rate(sample_rate, attack, T::one()),
        compute_rate(sample_rate, release, T::one()),
    )
}

fn compute_rate<T: DspFloat>(sample_rate: T, time: T, dist: T) -> T {
    if time > T::zero() {
        dist / (sample_rate * time)
    } else {
        T::zero()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ADSRParameters<T = f32> {
    pub attack: T,
    pub decay: T,
    pub sustain: T,
    pub release: T,
}

impl<T: DspFloat> Default for ADSRParameters<T> {
    fn default() -> Self {
        Self {
            attack: T::from_f32(0.1),
            decay: T::from_f32(0.1),
            sustain: T::from_f32(1.0),
            release: T::from_f32(0.1),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ADSRState {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Debug)]
pub struct ADSREnvelope<T = f32> {
    state: ADSRState,
    parameters: ADSRParameters<T>,
    current_level: T,
    sample_rate: T,
    attack_rate: T,
    decay_rate: T,
    release_rate: T,
}

impl<T: DspFloat> ADSREnvelope<T> {
    pub fn new(sample_rate: T, parameters: ADSRParameters<T>) -> Self {
        let (attack_rate, decay_rate, release_rate) = compute_adsr_rates(sample_rate, parameters);
        Self {
            state: ADSRState::Idle,
            sample_rate,
            parameters,
            current_level: T::zero(),
            attack_rate,
            decay_rate,
            release_rate,
        }
    }

    pub fn reset(&mut self) {
        self.state = ADSRState::Idle;
        self.current_level = T::zero();
    }

    pub fn note_on(&mut self) {
        if self.attack_rate > T::zero() {
            self.state = ADSRState::Attack;
        } else if self.decay_rate > T::zero() {
            self.state = ADSRState::Decay;
            self.current_level = T::one();
        } else {
            self.state = ADSRState::Sustain;
            self.current_level = self.parameters.sustain;
        }
    }

    pub fn note_off(&mut self) {
        if self.state != ADSRState::Idle {
            if self.parameters.release > T::zero() {
                self.state = ADSRState::Release;
                self.release_rate =
                    self.current_level / (self.parameters.release * self.sample_rate);
            } else {
                self.reset();
            }
        }
    }
}

fn compute_adsr_rates<T: DspFloat>(sample_rate: T, parameters: ADSRParameters<T>) -> (T, T, T) {
    (
        compute_rate(sample_rate, parameters.attack, T::one()),
        compute_rate(sample_rate, parameters.decay, T::one() - parameters.sustain),
        compute_rate(sample_rate, parameters.release, parameters.sustain),
    )
}
