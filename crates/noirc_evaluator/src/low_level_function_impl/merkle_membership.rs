use super::RuntimeError;
use super::{object_to_wit_bits, GadgetCaller};
use crate::interpreter::Interpreter;
use crate::object::{Array, Object};
use crate::Environment;
use acvm::acir::circuit::gate::{GadgetCall, GadgetInput, Gate};
use acvm::acir::OPCODE;
use acvm::FieldElement;
use noirc_frontend::hir_def::expr::HirCallExpression;

pub struct MerkleMembershipGadget;

impl GadgetCaller for MerkleMembershipGadget {
    fn name() -> OPCODE {
        OPCODE::MerkleMembership
    }

    fn call(
        evaluator: &mut Interpreter,
        env: &mut Environment,
        call_expr: HirCallExpression,
    ) -> Result<Object, RuntimeError> {
        let inputs = MerkleMembershipGadget::prepare_inputs(evaluator, env, call_expr)?;

        // Create a fresh variable which will be the boolean indicating
        // whether the item was in the tree or not

        let merkle_mem_witness = evaluator.add_witness_to_cs();
        let merkle_mem_object = Object::from_witness(merkle_mem_witness);

        let merkle_mem_gate = GadgetCall {
            name: MerkleMembershipGadget::name(),
            inputs,
            outputs: vec![merkle_mem_witness],
        };

        evaluator.push_gate(Gate::GadgetCall(merkle_mem_gate));

        Ok(merkle_mem_object)
    }
}

impl MerkleMembershipGadget {
    pub(super) fn prepare_inputs(
        evaluator: &mut Interpreter,
        env: &mut Environment,
        mut call_expr: HirCallExpression,
    ) -> Result<Vec<GadgetInput>, RuntimeError> {
        assert_eq!(call_expr.arguments.len(), 4);

        let hash_path = call_expr.arguments.pop().unwrap();
        let index = call_expr.arguments.pop().unwrap();
        let leaf = call_expr.arguments.pop().unwrap();
        let root = call_expr.arguments.pop().unwrap();

        let hash_path = Array::from_expression(evaluator, env, &hash_path)?;
        let index = evaluator.expression_to_object(env, &index)?;
        let leaf = evaluator.expression_to_object(env, &leaf)?;
        let root = evaluator.expression_to_object(env, &root)?;

        let index_witness = index.witness().unwrap();
        let leaf_witness = leaf.witness().unwrap();
        let root_witness = root.witness().unwrap();

        let mut inputs: Vec<GadgetInput> =
            vec![GadgetInput { witness: root_witness, num_bits: FieldElement::max_num_bits() }];

        inputs.push(GadgetInput { witness: leaf_witness, num_bits: FieldElement::max_num_bits() });

        inputs.push(GadgetInput { witness: index_witness, num_bits: FieldElement::max_num_bits() });

        for element in hash_path.contents.into_iter() {
            let gadget_inp = object_to_wit_bits(&element);
            assert_eq!(gadget_inp.num_bits, FieldElement::max_num_bits());
            inputs.push(gadget_inp);
        }

        Ok(inputs)
    }
}
