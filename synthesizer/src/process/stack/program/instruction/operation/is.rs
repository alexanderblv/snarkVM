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

use super::*;

impl<N: Network> Stack<N> {
    /// Evaluates the instruction.
    #[inline]
    pub fn evaluate_is<A: circuit::Aleo<Network = N>, const VARIANT: u8>(
        &self,
        is: &IsInstruction<N, VARIANT>,
        registers: &mut Registers<N, A>,
    ) -> Result<()> {
        // Ensure the number of operands is correct.
        if is.operands().len() != 2 {
            bail!(
                "Instruction '{}' expects 2 operands, found {} operands",
                IsInstruction::<N, VARIANT>::opcode(),
                is.operands().len()
            )
        }

        // Retrieve the inputs.
        let input_a = registers.load(self, &is.operands()[0])?;
        let input_b = registers.load(self, &is.operands()[1])?;

        // Check the inputs.
        let output = match VARIANT {
            0 => Literal::Boolean(Boolean::new(input_a == input_b)),
            1 => Literal::Boolean(Boolean::new(input_a != input_b)),
            _ => bail!("Invalid 'is' variant: {VARIANT}"),
        };
        // Store the output.
        registers.store(self, &is.destinations()[0], Value::Plaintext(Plaintext::from(output)))
    }

    /// Executes the instruction.
    #[inline]
    pub fn execute_is<A: circuit::Aleo<Network = N>, const VARIANT: u8>(
        &self,
        is: &IsInstruction<N, VARIANT>,
        registers: &mut Registers<N, A>,
    ) -> Result<()> {
        // Ensure the number of operands is correct.
        if is.operands().len() != 2 {
            bail!(
                "Instruction '{}' expects 2 operands, found {} operands",
                IsInstruction::opcode(),
                is.operands().len()
            )
        }

        // Retrieve the inputs.
        let input_a = registers.load_circuit(self, &is.operands()[0])?;
        let input_b = registers.load_circuit(self, &is.operands()[1])?;

        // Check the inputs.
        let output = match VARIANT {
            0 => circuit::Literal::Boolean(input_a.is_equal(&input_b)),
            1 => circuit::Literal::Boolean(input_a.is_not_equal(&input_b)),
            _ => bail!("Invalid 'is' variant: {VARIANT}"),
        };
        // Convert the output to a stack value.
        let output = circuit::Value::Plaintext(circuit::Plaintext::Literal(output, Default::default()));
        // Store the output.
        registers.store_circuit(self, &is.destinations()[0], output)
    }

    /// Returns the output type from the given program and input types.
    #[inline]
    pub fn is_output_types<const VARIANT: u8>(
        &self,
        is: &IsInstruction<N, VARIANT>,
        input_types: &[RegisterType<N>],
    ) -> Result<Vec<RegisterType<N>>> {
        // Ensure the number of input types is correct.
        if input_types.len() != 2 {
            bail!(
                "Instruction '{}' expects 2 inputs, found {} inputs",
                IsInstruction::<N, VARIANT>::opcode(),
                input_types.len()
            )
        }
        // Ensure the operands are of the same type.
        if input_types[0] != input_types[1] {
            bail!(
                "Instruction '{}' expects inputs of the same type. Found inputs of type '{}' and '{}'",
                IsInstruction::<N, VARIANT>::opcode(),
                input_types[0],
                input_types[1]
            )
        }
        // Ensure the number of operands is correct.
        if is.operands().len() != 2 {
            bail!(
                "Instruction '{}' expects 2 operands, found {} operands",
                IsInstruction::<N, VARIANT>::opcode(),
                is.operands().len()
            )
        }

        match VARIANT {
            0 | 1 => Ok(vec![RegisterType::Plaintext(PlaintextType::Literal(LiteralType::Boolean))]),
            _ => bail!("Invalid 'is' variant: {VARIANT}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ProvingKey, VerifyingKey};
    use circuit::AleoV0;
    use console::network::Testnet3;

    use std::collections::HashMap;

    type CurrentNetwork = Testnet3;
    type CurrentAleo = AleoV0;

    /// Samples the stack. Note: Do not replicate this for real program use, it is insecure.
    #[allow(clippy::type_complexity)]
    fn sample_stack(
        opcode: Opcode,
        type_a: LiteralType,
        type_b: LiteralType,
        mode_a: circuit::Mode,
        mode_b: circuit::Mode,
        cache: &mut HashMap<String, (ProvingKey<CurrentNetwork>, VerifyingKey<CurrentNetwork>)>,
    ) -> Result<(Stack<CurrentNetwork>, Vec<Operand<CurrentNetwork>>, Register<CurrentNetwork>)> {
        use crate::{Process, Program};
        use console::program::Identifier;

        // Initialize the opcode.
        let opcode = opcode.to_string();

        // Initialize the function name.
        let function_name = Identifier::<CurrentNetwork>::from_str("run")?;

        // Initialize the registers.
        let r0 = Register::Locator(0);
        let r1 = Register::Locator(1);
        let r2 = Register::Locator(2);

        // Initialize the program.
        let program = Program::from_str(&format!(
            "program testing.aleo;
            function {function_name}:
                input {r0} as {type_a}.{mode_a};
                input {r1} as {type_b}.{mode_b};
                {opcode} {r0} {r1} into {r2};
        "
        ))?;

        // Initialize the operands.
        let operand_a = Operand::Register(r0);
        let operand_b = Operand::Register(r1);
        let operands = vec![operand_a, operand_b];

        // Initialize the stack.
        let stack = Stack::new(&Process::load_with_cache(cache)?, &program)?;

        Ok((stack, operands, r2))
    }

    /// Samples the registers. Note: Do not replicate this for real program use, it is insecure.
    fn sample_registers(
        stack: &Stack<CurrentNetwork>,
        literal_a: &Literal<CurrentNetwork>,
        literal_b: &Literal<CurrentNetwork>,
        mode_a: Option<circuit::Mode>,
        mode_b: Option<circuit::Mode>,
    ) -> Result<Registers<CurrentNetwork, CurrentAleo>> {
        use crate::{Authorization, CallStack};
        use console::program::Identifier;

        // Initialize the function name.
        let function_name = Identifier::from_str("run")?;

        // Initialize the registers.
        let mut registers = Registers::<CurrentNetwork, CurrentAleo>::new(
            CallStack::evaluate(Authorization::new(&[]))?,
            stack.get_register_types(&function_name)?.clone(),
        );

        // Initialize the registers.
        let r0 = Register::Locator(0);
        let r1 = Register::Locator(1);

        // Initialize the console values.
        let value_a = Value::Plaintext(Plaintext::from(literal_a));
        let value_b = Value::Plaintext(Plaintext::from(literal_b));

        // Store the values in the console registers.
        registers.store(stack, &r0, value_a.clone())?;
        registers.store(stack, &r1, value_b.clone())?;

        if let (Some(mode_a), Some(mode_b)) = (mode_a, mode_b) {
            use circuit::Inject;

            // Initialize the circuit values.
            let circuit_a = circuit::Value::new(mode_a, value_a);
            let circuit_b = circuit::Value::new(mode_b, value_b);

            // Store the values in the circuit registers.
            registers.store_circuit(stack, &r0, circuit_a)?;
            registers.store_circuit(stack, &r1, circuit_b)?;
        }

        Ok(registers)
    }

    fn check_is<const VARIANT: u8>(
        operation: impl FnOnce(
            Vec<Operand<CurrentNetwork>>,
            Register<CurrentNetwork>,
        ) -> IsInstruction<CurrentNetwork, VARIANT>,
        opcode: Opcode,
        literal_a: &Literal<CurrentNetwork>,
        literal_b: &Literal<CurrentNetwork>,
        mode_a: &circuit::Mode,
        mode_b: &circuit::Mode,
        cache: &mut HashMap<String, (ProvingKey<CurrentNetwork>, VerifyingKey<CurrentNetwork>)>,
    ) {
        use circuit::Eject;

        println!("Checking '{opcode}' for '{literal_a}.{mode_a}' and '{literal_b}.{mode_b}'");

        // Initialize the types.
        let type_a = literal_a.to_type();
        let type_b = literal_b.to_type();
        assert_eq!(type_a, type_b, "The two literals must be the *same* type for this test");

        // Initialize the stack.
        let (stack, operands, destination) = sample_stack(opcode, type_a, type_b, *mode_a, *mode_b, cache).unwrap();
        // Initialize the operation.
        let operation = operation(operands, destination.clone());
        // Initialize a destination operand.
        let destination_operand = Operand::Register(destination);

        /* First, check the operation *succeeds* when both operands are `literal_a.mode_a`. */
        {
            // Attempt to compute the valid operand case.
            let mut registers = sample_registers(&stack, literal_a, literal_a, None, None).unwrap();
            operation.evaluate(&stack, &mut registers).unwrap();

            // Retrieve the output.
            let output_a = registers.load_literal(&stack, &destination_operand).unwrap();

            // Ensure the output is correct.
            if let Literal::Boolean(output_a) = output_a {
                match VARIANT {
                    0 => assert!(*output_a, "Instruction '{operation}' failed (console): {literal_a} {literal_a}"),
                    1 => assert!(
                        !*output_a,
                        "Instruction '{operation}' should have failed (console): {literal_a} {literal_a}"
                    ),
                    _ => panic!("Found an invalid 'is' variant in the test"),
                }
            } else {
                panic!("The output must be a boolean (console)");
            }

            // Attempt to compute the valid operand case.
            let mut registers = sample_registers(&stack, literal_a, literal_a, Some(*mode_a), Some(*mode_a)).unwrap();
            operation.execute::<CurrentAleo>(&stack, &mut registers).unwrap();

            // Retrieve the output.
            let output_b = registers.load_literal_circuit(&stack, &destination_operand).unwrap();

            // Ensure the output is correct.
            if let circuit::Literal::Boolean(output_b) = output_b {
                match VARIANT {
                    0 => assert!(
                        output_b.eject_value(),
                        "Instruction '{operation}' failed (circuit): {literal_a}.{mode_a} {literal_a}.{mode_a}"
                    ),
                    1 => assert!(
                        !output_b.eject_value(),
                        "Instruction '{operation}' should have failed (circuit): {literal_a}.{mode_a} {literal_a}.{mode_a}"
                    ),
                    _ => panic!("Found an invalid 'is' variant in the test"),
                }
            } else {
                panic!("The output must be a boolean (circuit)");
            }

            // Ensure the circuit is satisfied.
            match VARIANT {
                0 => assert!(
                    <CurrentAleo as circuit::Environment>::is_satisfied(),
                    "Instruction '{operation}' should be satisfied (circuit): {literal_a}.{mode_a} {literal_a}.{mode_a}"
                ),
                1 => assert!(
                    <CurrentAleo as circuit::Environment>::is_satisfied(),
                    "Instruction '{operation}' should be satisfied (circuit): {literal_a}.{mode_a} {literal_a}.{mode_a}"
                ),
                _ => panic!("Found an invalid 'is' variant in the test"),
            }

            // Reset the circuit.
            <CurrentAleo as circuit::Environment>::reset();
        }
        /* Next, check the mismatching literals *fail*. */
        if literal_a != literal_b {
            // Attempt to compute the valid operand case.
            let mut registers = sample_registers(&stack, literal_a, literal_b, None, None).unwrap();
            operation.evaluate(&stack, &mut registers).unwrap();

            // Retrieve the output.
            let output_a = registers.load_literal(&stack, &destination_operand).unwrap();

            // Ensure the output is correct.
            if let Literal::Boolean(output_a) = output_a {
                match VARIANT {
                    0 => assert!(
                        !*output_a,
                        "Instruction '{operation}' should have failed (console): {literal_a} {literal_b}"
                    ),
                    1 => assert!(*output_a, "Instruction '{operation}' failed (console): {literal_a} {literal_b}"),
                    _ => panic!("Found an invalid 'is' variant in the test"),
                }
            } else {
                panic!("The output must be a boolean (console)");
            }

            // Attempt to compute the valid operand case.
            let mut registers = sample_registers(&stack, literal_a, literal_b, Some(*mode_a), Some(*mode_b)).unwrap();
            operation.execute::<CurrentAleo>(&stack, &mut registers).unwrap();

            // Retrieve the output.
            let output_b = registers.load_literal_circuit(&stack, &destination_operand).unwrap();

            // Ensure the output is correct.
            if let circuit::Literal::Boolean(output_b) = output_b {
                match VARIANT {
                    0 => assert!(
                        !output_b.eject_value(),
                        "Instruction '{operation}' failed (circuit): {literal_a}.{mode_a} {literal_b}.{mode_b}"
                    ),
                    1 => assert!(
                        output_b.eject_value(),
                        "Instruction '{operation}' should have failed (circuit): {literal_a}.{mode_a} {literal_b}.{mode_b}"
                    ),
                    _ => panic!("Found an invalid 'is' variant in the test"),
                }
            } else {
                panic!("The output must be a boolean (circuit)");
            }

            // Ensure the circuit is correct.
            match VARIANT {
                0 => assert!(
                    <CurrentAleo as circuit::Environment>::is_satisfied(),
                    "Instruction '{operation}' should be satisfied (circuit): {literal_a}.{mode_a} {literal_b}.{mode_b}"
                ),
                1 => assert!(
                    <CurrentAleo as circuit::Environment>::is_satisfied(),
                    "Instruction '{operation}' should be satisfied (circuit): {literal_a}.{mode_a} {literal_b}.{mode_b}"
                ),
                _ => panic!("Found an invalid 'is' variant in the test"),
            }

            // Reset the circuit.
            <CurrentAleo as circuit::Environment>::reset();
        }
    }

    fn check_is_fails(
        opcode: Opcode,
        literal_a: &Literal<CurrentNetwork>,
        literal_b: &Literal<CurrentNetwork>,
        mode_a: &circuit::Mode,
        mode_b: &circuit::Mode,
        cache: &mut HashMap<String, (ProvingKey<CurrentNetwork>, VerifyingKey<CurrentNetwork>)>,
    ) {
        // Initialize the types.
        let type_a = literal_a.to_type();
        let type_b = literal_b.to_type();
        assert_ne!(type_a, type_b, "The two literals must be *different* types for this test");

        // If the types mismatch, ensure the stack fails to initialize.
        let result = sample_stack(opcode, type_a, type_b, *mode_a, *mode_b, cache);
        assert!(
            result.is_err(),
            "Stack should have failed to initialize for: {opcode} {type_a}.{mode_a} {type_b}.{mode_b}"
        );
    }

    #[test]
    fn test_is_eq_succeeds() {
        // Initialize the operation.
        let operation = |operands, destination| IsEq::<CurrentNetwork> { operands, destination };
        // Initialize the opcode.
        let opcode = IsEq::<CurrentNetwork>::opcode();

        // Prepare the rng.
        let mut rng = TestRng::default();

        // Prepare the test.
        let literals_a = crate::sample_literals!(CurrentNetwork, &mut rng);
        let literals_b = crate::sample_literals!(CurrentNetwork, &mut rng);
        let modes_a = [/* circuit::Mode::Constant, */ circuit::Mode::Public, circuit::Mode::Private];
        let modes_b = [/* circuit::Mode::Constant, */ circuit::Mode::Public, circuit::Mode::Private];

        // Prepare the key cache.
        let mut cache = Default::default();

        for (literal_a, literal_b) in literals_a.iter().zip_eq(literals_b.iter()) {
            for mode_a in &modes_a {
                for mode_b in &modes_b {
                    // Check the operation.
                    check_is(operation, opcode, literal_a, literal_b, mode_a, mode_b, &mut cache);
                }
            }
        }
    }

    #[test]
    fn test_is_eq_fails() {
        // Initialize the opcode.
        let opcode = IsEq::<CurrentNetwork>::opcode();

        // Prepare the rng.
        let mut rng = TestRng::default();

        // Prepare the test.
        let literals_a = crate::sample_literals!(CurrentNetwork, &mut rng);
        let literals_b = crate::sample_literals!(CurrentNetwork, &mut rng);
        let modes_a = [/* circuit::Mode::Constant, */ circuit::Mode::Public, circuit::Mode::Private];
        let modes_b = [/* circuit::Mode::Constant, */ circuit::Mode::Public, circuit::Mode::Private];

        // Prepare the key cache.
        let mut cache = Default::default();

        for literal_a in &literals_a {
            for literal_b in &literals_b {
                if literal_a.to_type() != literal_b.to_type() {
                    for mode_a in &modes_a {
                        for mode_b in &modes_b {
                            // Check the operation fails.
                            check_is_fails(opcode, literal_a, literal_b, mode_a, mode_b, &mut cache);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_is_neq_succeeds() {
        // Initialize the operation.
        let operation = |operands, destination| IsNeq::<CurrentNetwork> { operands, destination };
        // Initialize the opcode.
        let opcode = IsNeq::<CurrentNetwork>::opcode();

        // Prepare the rng.
        let mut rng = TestRng::default();

        // Prepare the test.
        let literals_a = crate::sample_literals!(CurrentNetwork, &mut rng);
        let literals_b = crate::sample_literals!(CurrentNetwork, &mut rng);
        let modes_a = [/* circuit::Mode::Constant, */ circuit::Mode::Public, circuit::Mode::Private];
        let modes_b = [/* circuit::Mode::Constant, */ circuit::Mode::Public, circuit::Mode::Private];

        // Prepare the key cache.
        let mut cache = Default::default();

        for (literal_a, literal_b) in literals_a.iter().zip_eq(literals_b.iter()) {
            for mode_a in &modes_a {
                for mode_b in &modes_b {
                    // Check the operation.
                    check_is(operation, opcode, literal_a, literal_b, mode_a, mode_b, &mut cache);
                }
            }
        }
    }

    #[test]
    fn test_is_neq_fails() {
        // Initialize the opcode.
        let opcode = IsNeq::<CurrentNetwork>::opcode();

        // Prepare the rng.
        let mut rng = TestRng::default();

        // Prepare the test.
        let literals_a = crate::sample_literals!(CurrentNetwork, &mut rng);
        let literals_b = crate::sample_literals!(CurrentNetwork, &mut rng);
        let modes_a = [/* circuit::Mode::Constant, */ circuit::Mode::Public, circuit::Mode::Private];
        let modes_b = [/* circuit::Mode::Constant, */ circuit::Mode::Public, circuit::Mode::Private];

        // Prepare the key cache.
        let mut cache = Default::default();

        for literal_a in &literals_a {
            for literal_b in &literals_b {
                if literal_a.to_type() != literal_b.to_type() {
                    for mode_a in &modes_a {
                        for mode_b in &modes_b {
                            // Check the operation fails.
                            check_is_fails(opcode, literal_a, literal_b, mode_a, mode_b, &mut cache);
                        }
                    }
                }
            }
        }
    }
}