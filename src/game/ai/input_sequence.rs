use crate::game::geometry::Rotation;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Hash, Default)]
pub struct InputSequence {
    lefts: u32,
    rights: u32,
    rotation: Rotation
}

impl InputSequence {
    pub fn new(lefts: u32, rights: u32, rotation: Rotation) -> Self {
        if lefts > 0 && rights > 0 {
            if lefts > rights {
                Self { lefts: lefts - rights, rights: 0, rotation }
            } else {
                Self { lefts: 0, rights: rights - lefts, rotation }
            }
        } else {
            Self { lefts, rights, rotation }
        }
    }

    pub fn into_left(self) -> Self {
        Self::new(self.lefts + 1, self.rights, self.rotation)
    }

    pub fn into_right(self) -> Self {
        Self::new(self.lefts, self.rights + 1, self.rotation)
    }

    pub fn into_rotation(self) -> Self {
        Self::new(self.lefts, self.rights, self.rotation.rotate(true))
    }

    pub fn lefts(&self) -> u32 {
        self.lefts
    }

    pub fn rights(&self) -> u32 {
        self.rights
    }

    pub fn rotations(&self) -> u32 {
        match self.rotation {
            Rotation::North => 0,
            Rotation::East => 1,
            Rotation::South => 2,
            Rotation::West => 3
        }
    }
}

impl Ord for InputSequence {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.lefts.cmp(&other.lefts)
            .then(self.rights.cmp(&other.rights))
            .then(self.rotation.cmp(&other.rotation))
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn rotates_180() {
        let seq = InputSequence::default()
            .into_rotation()
            .into_rotation();

        assert_eq!(seq, InputSequence { lefts: 0, rights: 0, rotation: Rotation::South });
        assert_eq!(seq.rotations(), 2);
    }

    #[test]
    fn rotates_360() {
        let seq = InputSequence::default()
            .into_rotation()
            .into_rotation()
            .into_rotation()
            .into_rotation();
        
        assert_eq!(seq, InputSequence { lefts: 0, rights: 0, rotation: Rotation::North });
        assert_eq!(seq.rotations(), 0);
    }


    #[test]
    fn deduplicates_movement() {
        let seq = InputSequence::default()
            .into_left()
            .into_left()
            .into_right();
        assert_eq!(seq, InputSequence { lefts: 1, rights: 0, rotation: Rotation::North });
    }
}