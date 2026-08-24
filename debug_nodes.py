import traceback
from swarm_v2.core.star_matrix_lattice import get_star_matrix_lattice
from swarm_v2.core.continuum_pbft import get_continuum_pbft

print("Testing Star Matrix Lattice register_node...")
try:
    sml = get_star_matrix_lattice()
    sml.register_node("test_n1", "Test Node", "Test lore", [1.0, 2.0], 2.0)
    lat = sml.get_full_lattice()
    print("Star Matrix Lattice OK:", len(lat["nodes"]))
except Exception as e:
    traceback.print_exc()

print("\nTesting PBFT cast_consensus_vote...")
try:
    pbft = get_continuum_pbft()
    vote = pbft.cast_consensus_vote("blk_101", "sha256_hash")
    print("PBFT Vote OK:", vote)
except Exception as e:
    traceback.print_exc()
