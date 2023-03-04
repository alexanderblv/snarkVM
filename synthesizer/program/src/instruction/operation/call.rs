// Copyright (C) 2019-2023 Aleo Systems Inc.
// This file is part of the snarkVM library.

// The snarkVM library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The snarkVM library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the snarkVM library. If not, see <https://www.gnu.org/licenses/>.

use crate::{Opcode, Operand};
use console::{
    network::prelude::*,
    program::{Identifier, Locator, Register},
};

/// The operator references a function name or closure name.
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum CallOperator<N: Network> {
    /// The reference to a non-local function or closure.
    Locator(Locator<N>),
    /// The reference to a local function or closure.
    Resource(Identifier<N>),
}

impl<N: Network> Parser for CallOperator<N> {
    /// Parses a string into an operator.
    #[inline]
    fn parse(string: &str) -> ParserResult<Self> {
        alt((map(Locator::parse, CallOperator::Locator), map(Identifier::parse, CallOperator::Resource)))(string)
    }
}

impl<N: Network> FromStr for CallOperator<N> {
    type Err = Error;

    /// Parses a string into an operator.
    #[inline]
    fn from_str(string: &str) -> Result<Self> {
        match Self::parse(string) {
            Ok((remainder, object)) => {
                // Ensure the remainder is empty.
                ensure!(remainder.is_empty(), "Failed to parse string. Found invalid character in: \"{remainder}\"");
                // Return the object.
                Ok(object)
            }
            Err(error) => bail!("Failed to parse string. {error}"),
        }
    }
}

impl<N: Network> Debug for CallOperator<N> {
    /// Prints the operator as a string.
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(self, f)
    }
}

impl<N: Network> Display for CallOperator<N> {
    /// Prints the operator to a string.
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            CallOperator::Locator(locator) => Display::fmt(locator, f),
            CallOperator::Resource(resource) => Display::fmt(resource, f),
        }
    }
}

impl<N: Network> FromBytes for CallOperator<N> {
    /// Reads the operation from a buffer.
    fn read_le<R: Read>(mut reader: R) -> IoResult<Self> {
        // Read the variant.
        let variant = u8::read_le(&mut reader)?;
        // Match the variant.
        match variant {
            0 => Ok(CallOperator::Locator(Locator::read_le(&mut reader)?)),
            1 => Ok(CallOperator::Resource(Identifier::read_le(&mut reader)?)),
            _ => Err(error("Failed to read CallOperator. Invalid variant.")),
        }
    }
}

impl<N: Network> ToBytes for CallOperator<N> {
    /// Writes the operation to a buffer.
    fn write_le<W: Write>(&self, mut writer: W) -> IoResult<()> {
        match self {
            CallOperator::Locator(locator) => {
                // Write the variant.
                0u8.write_le(&mut writer)?;
                // Write the locator.
                locator.write_le(&mut writer)
            }
            CallOperator::Resource(resource) => {
                // Write the variant.
                1u8.write_le(&mut writer)?;
                // Write the resource.
                resource.write_le(&mut writer)
            }
        }
    }
}

/// Calls the operands into the declared type.
/// i.e. `call transfer r0.owner 0u64 r1.amount into r1 r2;`
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Call<N: Network> {
    /// The reference.
    operator: CallOperator<N>,
    /// The operands.
    operands: Vec<Operand<N>>,
    /// The destination registers.
    destinations: Vec<Register<N>>,
}

impl<N: Network> Call<N> {
    /// Returns the opcode.
    #[inline]
    pub const fn opcode() -> Opcode {
        Opcode::Call
    }

    /// Return the operator.
    #[inline]
    pub const fn operator(&self) -> &CallOperator<N> {
        &self.operator
    }

    /// Returns the operands in the operation.
    #[inline]
    pub fn operands(&self) -> &[Operand<N>] {
        &self.operands
    }

    /// Returns the destination registers.
    #[inline]
    pub fn destinations(&self) -> Vec<Register<N>> {
        self.destinations.clone()
    }
}

impl<N: Network> Parser for Call<N> {
    /// Parses a string into an operation.
    #[inline]
    fn parse(string: &str) -> ParserResult<Self> {
        /// Parses an operand from the string.
        fn parse_operand<N: Network>(string: &str) -> ParserResult<Operand<N>> {
            // Parse the whitespace from the string.
            let (string, _) = Sanitizer::parse_whitespaces(string)?;
            // Parse the operand from the string.
            Operand::parse(string)
        }

        /// Parses a destination register from the string.
        fn parse_destination<N: Network>(string: &str) -> ParserResult<Register<N>> {
            // Parse the whitespace from the string.
            let (string, _) = Sanitizer::parse_whitespaces(string)?;
            // Parse the destination from the string.
            Register::parse(string)
        }

        // Parse the opcode from the string.
        let (string, _) = tag(*Self::opcode())(string)?;
        // Parse the whitespace from the string.
        let (string, _) = Sanitizer::parse_whitespaces(string)?;
        // Parse the name of the call from the string.
        let (string, operator) = CallOperator::parse(string)?;
        // Parse the whitespace from the string.
        let (string, _) = Sanitizer::parse_whitespaces(string)?;
        // Parse the operands from the string.
        let (string, operands) = map_res(many0(complete(parse_operand)), |operands: Vec<Operand<N>>| {
            // Ensure the number of operands is within the bounds.
            match operands.len() <= N::MAX_OPERANDS {
                true => Ok(operands),
                false => Err(error("Failed to parse 'call' opcode: too many operands")),
            }
        })(string)?;
        // Parse the whitespace from the string.
        let (string, _) = Sanitizer::parse_whitespaces(string)?;

        // Optionally parse the "into" from the string.
        let (string, destinations) = match opt(tag("into"))(string)? {
            // If the "into" was not parsed, return the string and an empty vector of destinations.
            (string, None) => (string, vec![]),
            // If the "into" was parsed, parse the destinations from the string.
            (string, Some(_)) => {
                // Parse the whitespace from the string.
                let (string, _) = Sanitizer::parse_whitespaces(string)?;
                // Parse the destinations from the string.
                let (string, destinations) =
                    map_res(many0(complete(parse_destination)), |destinations: Vec<Register<N>>| {
                        // Ensure the number of destinations is within the bounds.
                        match destinations.len() <= N::MAX_OPERANDS {
                            true => Ok(destinations),
                            false => Err(error("Failed to parse 'call' opcode: too many destinations")),
                        }
                    })(string)?;
                // Return the string and the destinations.
                (string, destinations)
            }
        };

        Ok((string, Self { operator, operands, destinations }))
    }
}

impl<N: Network> FromStr for Call<N> {
    type Err = Error;

    /// Parses a string into an operation.
    #[inline]
    fn from_str(string: &str) -> Result<Self> {
        match Self::parse(string) {
            Ok((remainder, object)) => {
                // Ensure the remainder is empty.
                ensure!(remainder.is_empty(), "Failed to parse string. Found invalid character in: \"{remainder}\"");
                // Return the object.
                Ok(object)
            }
            Err(error) => bail!("Failed to parse string. {error}"),
        }
    }
}

impl<N: Network> Debug for Call<N> {
    /// Prints the operation as a string.
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(self, f)
    }
}

impl<N: Network> Display for Call<N> {
    /// Prints the operation to a string.
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        // Ensure the number of operands is within the bounds.
        if self.operands.len() > N::MAX_OPERANDS {
            eprintln!("The number of operands must be <= {}", N::MAX_OPERANDS);
            return Err(fmt::Error);
        }
        // Ensure the number of destinations is within the bounds.
        if self.destinations.len() > N::MAX_OPERANDS {
            eprintln!("The number of destinations must be <= {}", N::MAX_OPERANDS);
            return Err(fmt::Error);
        }
        // Print the operation.
        write!(f, "{} {}", Self::opcode(), self.operator)?;
        self.operands.iter().try_for_each(|operand| write!(f, " {operand}"))?;
        if !self.destinations.is_empty() {
            write!(f, " into")?;
            self.destinations.iter().try_for_each(|destination| write!(f, " {destination}"))?;
        }
        Ok(())
    }
}

impl<N: Network> FromBytes for Call<N> {
    /// Reads the operation from a buffer.
    fn read_le<R: Read>(mut reader: R) -> IoResult<Self> {
        // Read the operator of the call.
        let operator = CallOperator::read_le(&mut reader)?;

        // Read the number of operands.
        let num_operands = u8::read_le(&mut reader)? as usize;
        // Ensure the number of operands is within the bounds.
        if num_operands > N::MAX_OPERANDS {
            return Err(error(format!("The number of operands must be <= {}", N::MAX_OPERANDS)));
        }

        // Initialize the vector for the operands.
        let mut operands = Vec::with_capacity(num_operands);
        // Read the operands.
        for _ in 0..num_operands {
            operands.push(Operand::read_le(&mut reader)?);
        }

        // Read the number of destination registers.
        let num_destinations = u8::read_le(&mut reader)? as usize;
        // Ensure the number of destinations is within the bounds.
        if num_destinations > N::MAX_OPERANDS {
            return Err(error(format!("The number of destinations must be <= {}", N::MAX_OPERANDS)));
        }

        // Initialize the vector for the destinations.
        let mut destinations = Vec::with_capacity(num_destinations);
        // Read the destination registers.
        for _ in 0..num_destinations {
            destinations.push(Register::read_le(&mut reader)?);
        }

        // Return the operation.
        Ok(Self { operator, operands, destinations })
    }
}

impl<N: Network> ToBytes for Call<N> {
    /// Writes the operation to a buffer.
    fn write_le<W: Write>(&self, mut writer: W) -> IoResult<()> {
        // Ensure the number of operands is within the bounds.
        if self.operands.len() > N::MAX_OPERANDS {
            return Err(error(format!("The number of operands must be <= {}", N::MAX_OPERANDS)));
        }
        // Ensure the number of destinations is within the bounds.
        if self.destinations.len() > N::MAX_OPERANDS {
            return Err(error(format!("The number of destinations must be <= {}", N::MAX_OPERANDS)));
        }

        // Write the name of the call.
        self.operator.write_le(&mut writer)?;
        // Write the number of operands.
        (self.operands.len() as u8).write_le(&mut writer)?;
        // Write the operands.
        self.operands.iter().try_for_each(|operand| operand.write_le(&mut writer))?;
        // Write the number of destination register.
        (self.destinations.len() as u8).write_le(&mut writer)?;
        // Write the destination registers.
        self.destinations.iter().try_for_each(|destination| destination.write_le(&mut writer))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use console::{
        network::Testnet3,
        program::{Address, Identifier, Literal, U64},
    };

    type CurrentNetwork = Testnet3;

    const TEST_CASES: &[&str] = &[
        "call foo",
        "call foo r0",
        "call foo r0.owner",
        "call foo r0 r1",
        "call foo r0.owner r0.gates",
        "call foo into r0",
        "call foo into r0 r1",
        "call foo into r0 r1 r2",
        "call foo r0 into r1",
        "call foo r0 r1 into r2",
        "call foo r0 r1 into r2 r3",
        "call foo r0 r1 r2 into r3 r4",
        "call foo r0 r1 r2 into r3 r4 r5",
    ];

    fn check_parser(
        string: &str,
        expected_operator: CallOperator<CurrentNetwork>,
        expected_operands: Vec<Operand<CurrentNetwork>>,
        expected_destinations: Vec<Register<CurrentNetwork>>,
    ) {
        // Check that the parser works.
        let (string, call) = Call::<CurrentNetwork>::parse(string).unwrap();

        // Check that the entire string was consumed.
        assert!(string.is_empty(), "Parser did not consume all of the string: '{string}'");

        // Check that the operator is correct.
        assert_eq!(call.operator, expected_operator, "The call operator is incorrect");

        // Check that the operands are correct.
        assert_eq!(call.operands.len(), expected_operands.len(), "The number of operands is incorrect");
        for (i, (given, expected)) in call.operands.iter().zip(expected_operands.iter()).enumerate() {
            assert_eq!(given, expected, "The {i}-th operand is incorrect");
        }

        // Check that the destinations are correct.
        assert_eq!(call.destinations.len(), expected_destinations.len(), "The number of destinations is incorrect");
        for (i, (given, expected)) in call.destinations.iter().zip(expected_destinations.iter()).enumerate() {
            assert_eq!(given, expected, "The {i}-th destination is incorrect");
        }
    }

    #[test]
    fn test_parse() {
        check_parser(
            "call transfer r0.owner r0.gates r0.token_amount into r1 r2 r3",
            CallOperator::from_str("transfer").unwrap(),
            vec![
                Operand::Register(Register::Member(0, vec![Identifier::from_str("owner").unwrap()])),
                Operand::Register(Register::Member(0, vec![Identifier::from_str("gates").unwrap()])),
                Operand::Register(Register::Member(0, vec![Identifier::from_str("token_amount").unwrap()])),
            ],
            vec![Register::Locator(1), Register::Locator(2), Register::Locator(3)],
        );

        check_parser(
            "call mint_public aleo1wfyyj2uvwuqw0c0dqa5x70wrawnlkkvuepn4y08xyaqfqqwweqys39jayw 100u64",
            CallOperator::from_str("mint_public").unwrap(),
            vec![
                Operand::Literal(Literal::Address(
                    Address::from_str("aleo1wfyyj2uvwuqw0c0dqa5x70wrawnlkkvuepn4y08xyaqfqqwweqys39jayw").unwrap(),
                )),
                Operand::Literal(Literal::U64(U64::from_str("100u64").unwrap())),
            ],
            vec![],
        );

        check_parser(
            "call get_magic_number into r0",
            CallOperator::from_str("get_magic_number").unwrap(),
            vec![],
            vec![Register::Locator(0)],
        );

        check_parser("call noop", CallOperator::from_str("noop").unwrap(), vec![], vec![])
    }

    #[test]
    fn test_display() {
        for expected in TEST_CASES {
            assert_eq!(Call::<CurrentNetwork>::from_str(expected).unwrap().to_string(), *expected);
        }
    }

    #[test]
    fn test_bytes() {
        for case in TEST_CASES {
            let expected = Call::<CurrentNetwork>::from_str(case).unwrap();

            // Check the byte representation.
            let expected_bytes = expected.to_bytes_le().unwrap();
            assert_eq!(expected, Call::read_le(&expected_bytes[..]).unwrap());
            assert!(Call::<CurrentNetwork>::read_le(&expected_bytes[1..]).is_err());
        }
    }
}