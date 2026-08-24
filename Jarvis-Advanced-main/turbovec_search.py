import sys
import json
import numpy as np
import turbovec

def main():
    try:
        # Read inputs from stdin
        input_data = json.loads(sys.stdin.read())
        
        query_vector = np.array(input_data["query_vector"], dtype=np.float32)
        items = input_data["items"]
        
        if not items:
            print(json.dumps([]))
            return
            
        vectors = []
        ids = []
        id_to_item = {}
        
        for idx, item in enumerate(items):
            vectors.append(item["vector"])
            ids.append(idx)
            id_to_item[idx] = item
            
        vectors = np.array(vectors, dtype=np.float32)
        ids = np.array(ids, dtype=np.uint64)
        
        dim = vectors.shape[1]
        
        # Normalize vectors to unit length
        # Cosine Similarity is equivalent to Inner Product (dot product) of normalized unit vectors
        # For unit vectors, L2 distance squared is: ||u - v||^2 = 2 - 2 * CosineSimilarity(u, v)
        # Therefore, minimizing L2 distance of normalized vectors directly maximizes Cosine Similarity.
        norms = np.linalg.norm(vectors, axis=1, keepdims=True)
        norms[norms == 0] = 1.0
        normalized_vectors = vectors / norms
        
        q_norm = np.linalg.norm(query_vector)
        normalized_query = query_vector / (q_norm if q_norm > 0 else 1.0)
        
        # Construct and query turbovec IdMapIndex
        index = turbovec.IdMapIndex(dim=dim, bit_width=4)
        index.add_with_ids(normalized_vectors, ids)
        index.prepare()
        
        k = min(len(items), 10)
        # Search returns (scores, ids)
        scores, matched_ids = index.search(np.array([normalized_query], dtype=np.float32), k=k)
        
        results = []
        for score, matched_id in zip(scores[0], matched_ids[0]):
            # Cast matched_id to int to access local dictionary
            m_id = int(matched_id)
            if m_id in id_to_item:
                item = id_to_item[m_id]
                # Calculate exact Cosine Similarity on the float vectors for precision sorting
                dot_product = float(np.dot(normalized_query, normalized_vectors[m_id]))
                results.append({
                    "id": item.get("id"),
                    "description": item.get("description"),
                    "timestamp": item.get("timestamp"),
                    "importance": item.get("importance"),
                    "similarity": dot_product
                })
                
        # Sort results by similarity descending
        results.sort(key=lambda x: x["similarity"], reverse=True)
        print(json.dumps(results))
        
    except Exception as e:
        # Return fallback empty JSON array and log error to stderr
        print(f"Error in turbovec_search: {e}", file=sys.stderr)
        print(json.dumps([]))

if __name__ == "__main__":
    main()
