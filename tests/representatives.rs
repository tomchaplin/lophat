use std::collections::HashMap;

use lophat::{
	algorithms::{
		Decomposition,
		DecompositionAlgo,
		LockFreeAlgorithm,
		LockingAlgorithm,
		NoVMatrixError,
		SerialAlgorithm,
	},
	columns::{BitSetColumn, BitSetVecHybridColumn, Column, VecColumn},
	options::LoPhatOptions,
	utils::{
		ExtendedUsize::{Finite, Infinity},
		PersistenceDiagram,
	},
};

fn representative_matrix() -> Vec<VecColumn> {
	// Finite and essential 1-cycles, plus an essential 2-cycle. In serial
	// reduction without clearing, R[9] differs from V[6], distinguishing
	// the prescribed choice of R at death from V at birth.
	vec![
		(0, vec![]),
		(0, vec![]),
		(0, vec![]),
		(1, vec![0, 1]),
		(1, vec![0, 2]),
		(1, vec![1, 2]),
		(1, vec![0, 1]),
		(1, vec![0, 2]),
		(2, vec![3, 4, 5]),
		(2, vec![4, 5, 6]),
		(2, vec![3, 6]),
	]
	.into_iter()
	.map(VecColumn::from)
	.collect()
}

fn chain_mask(indices: impl Iterator<Item = usize>) -> u64 {
	indices.fold(0, |mask, index| mask ^ (1 << index))
}

fn reduce_mask(mut chain: u64, basis: &[u64; 64]) -> u64 {
	while chain != 0 {
		let pivot = 63 - chain.leading_zeros() as usize;
		if basis[pivot] == 0 {
			break;
		}
		chain ^= basis[pivot];
	}
	chain
}

fn is_boundary_at(matrix: &[VecColumn], chain: u64, filtration_index: usize) -> bool {
	// Independent GF(2) elimination on the original boundary columns,
	// without using the algorithm's R or V matrices.
	let mut basis = [0; 64];
	for column in matrix.iter().take(filtration_index + 1) {
		let reduced = reduce_mask(chain_mask(column.entries()), &basis);
		if reduced != 0 {
			basis[63 - reduced.leading_zeros() as usize] = reduced;
		}
	}
	reduce_mask(chain, &basis) == 0
}

fn verify_representatives<C: Column>(decomposition: &impl Decomposition<C>, matrix: &[VecColumn]) {
	let expected: PersistenceDiagram = [
		(0, Infinity),
		(1, Finite(3)),
		(2, Finite(4)),
		(5, Finite(8)),
		(6, Finite(9)),
		(7, Infinity),
		(10, Infinity),
	]
	.into_iter()
	.collect();
	let (diagram, representatives) = decomposition.diagram_with_reps();
	assert_eq!(diagram, expected);
	assert_eq!(diagram, decomposition.diagram());
	let representatives = representatives.unwrap();
	assert_eq!(representatives.len(), diagram.len());
	for (&birth, &death) in diagram.iter() {
		let representative = &representatives[&birth];
		let chain = chain_mask(representative.iter().copied());
		assert_eq!(chain.count_ones() as usize, representative.len());
		assert_eq!(representative.iter().max(), Some(&birth));
		assert!(
			representative
				.iter()
				.all(|&index| matrix[index].degree() == matrix[birth].degree())
		);
		let boundary = representative
			.iter()
			.fold(0, |mask, &index| mask ^ chain_mask(matrix[index].entries()));
		assert_eq!(
			boundary, 0,
			"representative at birth {birth} must be a cycle"
		);
		for filtration_index in birth..matrix.len() {
			let should_be_boundary = match death {
				Finite(death) => filtration_index >= death,
				Infinity => false,
			};
			assert_eq!(
				is_boundary_at(matrix, chain, filtration_index),
				should_be_boundary,
				"birth {birth}, filtration index {filtration_index}"
			);
		}
	}
}

fn check_representatives<C: Column, A: DecompositionAlgo<C, Options = LoPhatOptions>>() {
	let matrix = representative_matrix();
	let thread_counts: &[usize] = if cfg!(feature = "local_thread_pool") {
		&[1, 2]
	} else {
		&[0]
	};
	for clearing in [false, true] {
		for &num_threads in thread_counts {
			let options = LoPhatOptions {
				maintain_v: true,
				clearing,
				num_threads,
				..Default::default()
			};
			let columns = matrix.iter().map(|column| {
				let mut converted = C::new_with_degree(column.degree());
				converted.add_entries(column.entries());
				converted
			});
			let decomposition = A::init(Some(options)).add_cols(columns).decompose();
			verify_representatives(&decomposition, &matrix);
			#[cfg(feature = "serde")]
			verify_representatives(
				&lophat::utils::clone_to_file_format(&decomposition),
				&matrix,
			);
		}
	}
}

#[test]
fn serial_representatives_are_persistent_cycles() {
	check_representatives::<VecColumn, SerialAlgorithm<VecColumn>>();
	check_representatives::<BitSetColumn, SerialAlgorithm<BitSetColumn>>();
	check_representatives::<BitSetVecHybridColumn, SerialAlgorithm<BitSetVecHybridColumn>>();
}

#[test]
fn serial_uses_r_at_death_and_v_at_birth() {
	let options = LoPhatOptions {
		maintain_v: true,
		clearing: false,
		..Default::default()
	};
	let decomposition = SerialAlgorithm::init(Some(options))
		.add_cols(representative_matrix().into_iter())
		.decompose();
	let (_, representatives) = decomposition.diagram_with_reps();
	assert_eq!(
		representatives.unwrap(),
		HashMap::from([
			(0, vec![0]),
			(1, vec![0, 1]),
			(2, vec![0, 2]),
			(5, vec![3, 4, 5]),
			(6, vec![4, 5, 6]),
			(7, vec![4, 7]),
			(10, vec![8, 9, 10]),
		])
	);
	// A valid representative from V at a finite birth would be a different
	// choice from the method's promised representative from R at death.
	assert_eq!(
		decomposition
			.get_v_col(6)
			.unwrap()
			.entries()
			.collect::<Vec<_>>(),
		vec![3, 6]
	);
}

#[test]
fn lockfree_representatives_are_persistent_cycles() {
	check_representatives::<VecColumn, LockFreeAlgorithm<VecColumn>>();
	check_representatives::<BitSetColumn, LockFreeAlgorithm<BitSetColumn>>();
	check_representatives::<BitSetVecHybridColumn, LockFreeAlgorithm<BitSetVecHybridColumn>>();
}

#[test]
fn locking_representatives_are_persistent_cycles() {
	check_representatives::<VecColumn, LockingAlgorithm<VecColumn>>();
	check_representatives::<BitSetColumn, LockingAlgorithm<BitSetColumn>>();
	check_representatives::<BitSetVecHybridColumn, LockingAlgorithm<BitSetVecHybridColumn>>();
}

fn check_representative_errors<A: DecompositionAlgo<VecColumn, Options = LoPhatOptions>>() {
	for maintain_v in [false, true] {
		let options = LoPhatOptions {
			maintain_v,
			num_threads: if cfg!(feature = "local_thread_pool") {
				1
			} else {
				0
			},
			..Default::default()
		};
		let empty = A::init(Some(options)).decompose();
		assert_eq!(
			empty.diagram_with_reps(),
			(
				PersistenceDiagram::default(),
				Err(NoVMatrixError::EmptyDecompositionError)
			)
		);
		let nonempty = A::init(Some(options))
			.add_cols(std::iter::once(VecColumn::from((0, vec![]))))
			.decompose();
		let (diagram, representatives) = nonempty.diagram_with_reps();
		assert_eq!(diagram, [(0, Infinity)].into_iter().collect());
		if maintain_v {
			assert_eq!(representatives, Ok(HashMap::from([(0, vec![0])])));
		} else {
			assert_eq!(representatives, Err(NoVMatrixError::VMatrixDiscardedError));
			// V is required by this API even when all features are finite.
			let finite_only = A::init(Some(options))
				.add_cols([(1, vec![]), (2, vec![0])].into_iter().map(VecColumn::from))
				.decompose();
			let (diagram, representatives) = finite_only.diagram_with_reps();
			assert_eq!(diagram, [(0, Finite(1))].into_iter().collect());
			assert_eq!(representatives, Err(NoVMatrixError::VMatrixDiscardedError));
		}
	}
}

#[test]
fn serial_representative_errors_preserve_diagram() {
	check_representative_errors::<SerialAlgorithm<VecColumn>>();
}

#[test]
fn lockfree_representative_errors_preserve_diagram() {
	check_representative_errors::<LockFreeAlgorithm<VecColumn>>();
}

#[test]
fn locking_representative_errors_preserve_diagram() {
	check_representative_errors::<LockingAlgorithm<VecColumn>>();
}
