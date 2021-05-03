use crate::blocks::Number;

pub struct Timer {
    period: Number,
    current: Number,
}

impl Timer {
    pub fn new(period: Number) -> Timer {
        Timer {
            period,
            current: period + 1,
        }
    }

    pub fn tick(&mut self) {
        if self.current <= self.period {
            self.current += 1;
        }
    }

    pub fn is_triggered(&self) -> bool {
        self.current == self.period
    }

    pub fn is_not_triggered_yet(&self) -> bool {
        self.current < self.period
    }

    pub fn is_started(&self) -> bool {
        self.is_not_triggered_yet() || self.is_triggered()
    }

    pub fn start(&mut self) {
        self.current = 0;
    }

    pub fn stop(&mut self) {
        self.current = self.period + 1;
    }
}

pub struct BlinkAnimation {
    timer: Timer,
    changes_remain: Number,
    show: bool,
}

impl BlinkAnimation {
    pub fn new() -> BlinkAnimation {
        BlinkAnimation {
            timer: Timer::new(15),
            changes_remain: 0,
            show: true,
        }
    }

    pub fn start(&mut self) {
        self.changes_remain = 6;
        self.show = false;
        self.timer.start();
    }

    pub fn stop(&mut self) {
        self.timer.stop();
        self.changes_remain = 0;
        self.show = true;
    }

    pub fn tick(&mut self) {
        self.timer.tick();

        if self.timer.is_triggered() {
            self.show = !self.show;
            self.changes_remain -= 1;

            if self.changes_remain > 0 {
                self.timer.start();
            } else {
                self.show = true;
            }
        }
    }

    pub fn is_show(&self) -> bool {
        self.show
    }

    pub fn is_triggered(&self) -> bool {
        self.changes_remain == 0 && self.timer.is_triggered()
    }
    pub fn is_not_triggered_yet(&self) -> bool {
        self.changes_remain != 0 || self.timer.is_not_triggered_yet()
    }
    pub fn is_started(&self) -> bool {
        self.is_not_triggered_yet() || self.is_triggered()
    }
}
