a minimal dua python interface that uses PyO3 + maturin to use the dua rust library.

This is the better route if you want the “index-like” structure.

dua represents traversal as an in-memory StableGraph<EntryData, (), Directed>, with TreeIndex node IDs.

The binding does traversals, integrate events into the graph, then converts the graph into python objects etc.