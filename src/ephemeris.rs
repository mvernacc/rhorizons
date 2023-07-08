use crate::utilities::{take_expecting, take_or_empty};

/// Position (in km) and velocity (in km/s) of a body.
#[derive(Debug, PartialEq)]
pub struct EphemerisVectorItem {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
}

#[derive(Debug, PartialEq)]
pub struct EphemerisOrbitalElementsItem {
    pub eccentricity: f32, //EC     Eccentricity, e
    //pub periapsis_distance: f32, //QR     Periapsis distance, q (km)
    pub inclination: f32, //IN     Inclination w.r.t X-Y plane, i (degrees)

    pub longitude_of_ascending_node: f32, //OM     Longitude of Ascending Node, OMEGA, (degrees)
    pub argument_of_perifocus: f32,       //W      Argument of Perifocus, w (degrees)
    //pub time_of_periapsis: f32,  //Tp     Time of periapsis (Julian Day Number)

    //pub mean_motion: f32,  //N      Mean motion, n (degrees/sec)
    pub mean_anomaly: f32, //MA     Mean anomaly, M (degrees)
    //pub true_anomaly: f32,  //TA     True anomaly, nu (degrees)
    pub semi_major_axis: f32, //A      Semi-major axis, a (km)
                              //pub apoapsis_distance: f32,  //AD     Apoapsis distance (km)
                              //pub siderral_orbit_period: f32  //PR     Sidereal orbit period (sec)
}

enum EphemerisVectorParserState {
    WaitingForSoe,
    WaitingForDate,
    WaitingForPosition,
    Position {
        position: [f32; 3],
    },
    Complete {
        position: [f32; 3],
        velocity: [f32; 3],
    },
    End,
}

enum EphemerisOrbitalElementsParserState {
    WaitingForSoe,
    WaitingForDate,
    WaitingForEccentricityAndInclination,
    EccentricityAndInclination {
        eccentricity: f32,
        inclination: f32,
    },
    AddedAscendingNodeAndPericfocus {
        eccentricity: f32,
        inclination: f32,
        longitude_of_ascending_node: f32,
        argument_of_perifocus: f32,
    },
    AddedMeanAnomaly {
        eccentricity: f32,
        inclination: f32,
        longitude_of_ascending_node: f32,
        argument_of_perifocus: f32,
        mean_anomaly: f32,
    },
    End,
}

pub struct EphemerisVectorParser<'a, Input: Iterator<Item = &'a str>> {
    state: EphemerisVectorParserState,
    input: Input,
}

pub struct EphemerisOrbitalElementsParser<'a, Input: Iterator<Item = &'a str>> {
    state: EphemerisOrbitalElementsParserState,
    input: Input,
}

impl<'a, Input: Iterator<Item = &'a str>> EphemerisVectorParser<'a, Input> {
    pub fn parse(input: Input) -> Self {
        Self {
            state: EphemerisVectorParserState::WaitingForSoe,
            input,
        }
    }
}

impl<'a, Input: Iterator<Item = &'a str>> EphemerisOrbitalElementsParser<'a, Input> {
    pub fn parse(input: Input) -> Self {
        Self {
            state: EphemerisOrbitalElementsParserState::WaitingForSoe,
            input,
        }
    }
}

impl<'a, Input: Iterator<Item = &'a str>> Iterator for EphemerisVectorParser<'a, Input> {
    type Item = EphemerisVectorItem;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(line) = self.input.next() {
                match self.state {
                    EphemerisVectorParserState::WaitingForSoe => {
                        if line == "$$SOE" {
                            self.state = EphemerisVectorParserState::WaitingForDate;
                        }
                    }
                    EphemerisVectorParserState::WaitingForDate => {
                        if line == "$$EOE" {
                            self.state = EphemerisVectorParserState::End;
                        } else {
                            self.state = EphemerisVectorParserState::WaitingForPosition;
                        }
                    }
                    EphemerisVectorParserState::WaitingForPosition => {
                        // TODO: Don't panic.
                        let line = take_expecting(line, " X =").unwrap();
                        let (x, line) = take_or_empty(line, 22);

                        let line = take_expecting(line, " Y =").unwrap();
                        let (y, line) = take_or_empty(line, 22);

                        let line = take_expecting(line, " Z =").unwrap();
                        let (z, _) = take_or_empty(line, 22);

                        self.state = EphemerisVectorParserState::Position {
                            position: [
                                x.trim().parse::<f32>().unwrap(),
                                y.trim().parse::<f32>().unwrap(),
                                z.trim().parse::<f32>().unwrap(),
                            ],
                        };
                    }
                    EphemerisVectorParserState::Position { position } => {
                        // TODO: Don't panic.
                        let line = take_expecting(line, " VX=").unwrap();
                        let (vx, line) = take_or_empty(line, 22);

                        let line = take_expecting(line, " VY=").unwrap();
                        let (vy, line) = take_or_empty(line, 22);

                        let line = take_expecting(line, " VZ=").unwrap();
                        let (vz, _) = take_or_empty(line, 22);

                        self.state = EphemerisVectorParserState::Complete {
                            position,
                            velocity: [
                                vx.trim().parse::<f32>().unwrap(),
                                vy.trim().parse::<f32>().unwrap(),
                                vz.trim().parse::<f32>().unwrap(),
                            ],
                        };
                    }
                    // Would parse third line and then return Item => ignores third line and returns directly
                    EphemerisVectorParserState::Complete { position, velocity } => {
                        self.state = EphemerisVectorParserState::WaitingForDate;
                        return Some(EphemerisVectorItem { position, velocity });
                    }
                    EphemerisVectorParserState::End => {
                        // Should we drain input iterator?
                        return None;
                    }
                }
            } else {
                // Input iterator is drained. Nothing to do.
                return None;
            }
        }
    }
}

impl<'a, Input: Iterator<Item = &'a str>> Iterator for EphemerisOrbitalElementsParser<'a, Input> {
    type Item = EphemerisOrbitalElementsItem;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(line) = self.input.next() {
                match self.state {
                    EphemerisOrbitalElementsParserState::WaitingForSoe => {
                        if line == "$$SOE" {
                            self.state = EphemerisOrbitalElementsParserState::WaitingForDate;
                        }
                    }
                    EphemerisOrbitalElementsParserState::WaitingForDate => {
                        if line == "$$EOE" {
                            self.state = EphemerisOrbitalElementsParserState::End;
                        } else {
                            self.state = EphemerisOrbitalElementsParserState::WaitingForEccentricityAndInclination;
                        }
                    }
                    EphemerisOrbitalElementsParserState::WaitingForEccentricityAndInclination => {
                        let line = take_expecting(line, " EC=").unwrap();
                        let (eccentricity, line) = take_or_empty(line, 22);

                        let line = take_expecting(line, " QR=").unwrap();
                        let (_periapsis_distance, line) = take_or_empty(line, 22);

                        let line = take_expecting(line, " IN=").unwrap();
                        let (inclination, _) = take_or_empty(line, 22);

                        self.state =
                            EphemerisOrbitalElementsParserState::EccentricityAndInclination {
                                eccentricity: eccentricity.trim().parse::<f32>().unwrap(),
                                inclination: inclination.trim().parse::<f32>().unwrap(),
                            };
                    }
                    EphemerisOrbitalElementsParserState::EccentricityAndInclination {
                        eccentricity,
                        inclination,
                    } => {
                        let line = take_expecting(line, " OM=").unwrap();
                        let (longitude_of_ascending_node, line) = take_or_empty(line, 22);

                        let line = take_expecting(line, " W =").unwrap();
                        let (argument_of_perifocus, line) = take_or_empty(line, 22);

                        let line = take_expecting(line, " Tp=").unwrap();
                        let (_time_of_periapsis, _) = take_or_empty(line, 22);

                        self.state =
                            EphemerisOrbitalElementsParserState::AddedAscendingNodeAndPericfocus {
                                eccentricity,
                                inclination,
                                longitude_of_ascending_node: longitude_of_ascending_node
                                    .trim()
                                    .parse::<f32>()
                                    .unwrap(),
                                argument_of_perifocus: argument_of_perifocus
                                    .trim()
                                    .parse::<f32>()
                                    .unwrap(),
                            };
                    }
                    EphemerisOrbitalElementsParserState::AddedAscendingNodeAndPericfocus {
                        eccentricity,
                        inclination,
                        longitude_of_ascending_node,
                        argument_of_perifocus,
                    } => {
                        let line = take_expecting(line, " N =").unwrap();
                        let (_mean_motion, line) = take_or_empty(line, 22);

                        let line = take_expecting(line, " MA=").unwrap();
                        let (mean_anomaly, line) = take_or_empty(line, 22);

                        let line = take_expecting(line, " TA=").unwrap();
                        let (_true_anomaly, _) = take_or_empty(line, 22);

                        self.state = EphemerisOrbitalElementsParserState::AddedMeanAnomaly {
                            eccentricity,
                            inclination,
                            longitude_of_ascending_node,
                            argument_of_perifocus,
                            mean_anomaly: mean_anomaly.trim().parse::<f32>().unwrap(),
                        };
                    }
                    // Parses last line and return Item
                    EphemerisOrbitalElementsParserState::AddedMeanAnomaly {
                        eccentricity,
                        inclination,
                        longitude_of_ascending_node,
                        argument_of_perifocus,
                        mean_anomaly,
                    } => {
                        let line = take_expecting(line, " A =").unwrap();
                        let (semi_major_axis, line) = take_or_empty(line, 22);

                        let line = take_expecting(line, " AD=").unwrap();
                        let (_apoapsis_distance, line) = take_or_empty(line, 22);

                        let line = take_expecting(line, " PR=").unwrap();
                        let (_siderral_orbit_period, _) = take_or_empty(line, 22);

                        self.state = EphemerisOrbitalElementsParserState::WaitingForDate;
                        return Some(EphemerisOrbitalElementsItem {
                            eccentricity,
                            inclination,
                            longitude_of_ascending_node,
                            argument_of_perifocus,
                            mean_anomaly,
                            semi_major_axis: semi_major_axis.trim().parse::<f32>().unwrap(),
                        });
                    }
                    EphemerisOrbitalElementsParserState::End => {
                        // Should we drain input iterator?
                        return None;
                    }
                }
            } else {
                // Input iterator is drained. Nothing to do.
                return None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsing_ephemeris_vector() {
        let data = include_str!("vector.txt");
        let ephem: Vec<_> = EphemerisVectorParser::parse(data.lines()).collect();
        assert_eq!(4, ephem.len());
        // TODO: This will probably fail intermittently due to float comparison.
        assert_eq!(
            EphemerisVectorItem {
                position: [
                    1.870010427985840E+02,
                    2.484687803242536E+03,
                    -5.861602653492581E+03
                ],

                velocity: [
                    -3.362664133558439E-01,
                    1.344100266143978E-02,
                    -5.030275220358716E-03
                ]
            },
            ephem[0]
        );
    }

    #[test]
    fn test_parsing_ephemeris_orbital_elements() {
        let data = include_str!("orbital_elements.txt");
        let ephem: Vec<_> = EphemerisOrbitalElementsParser::parse(data.lines()).collect();
        assert_eq!(4, ephem.len());
        // TODO: This will probably fail intermittently due to float comparison.
        assert_eq!(
            EphemerisOrbitalElementsItem {
                eccentricity: 1.711794334680415E-02,
                inclination: 3.134746902320420E-03,
                longitude_of_ascending_node: 1.633896137466430E+02,
                argument_of_perifocus: 3.006492364709574E+02,
                mean_anomaly: 1.635515780663357E+02,
                semi_major_axis: 1.495485150384278E+08,
            },
            ephem[0]
        );
    }
}