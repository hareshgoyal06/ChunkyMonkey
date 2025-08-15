def simple_chunk(text: str, size: int = 800, overlap: int = 150) -> list[str]:
    """
    Simple sliding window text chunking
    
    Args:
        text: Input text to chunk
        size: Maximum chunk size in characters
        overlap: Overlap between chunks in characters
        
    Returns:
        List of text chunks
    """
    if not text:
        return []
    
    chunks = []
    start = 0
    
    while start < len(text):
        end = start + size
        
        # If this isn't the last chunk, try to break at a word boundary
        if end < len(text):
            # Look for the last space or punctuation in the chunk
            last_space = text.rfind(' ', start, end)
            if last_space > start + size // 2:  # Only break at word if it's not too early
                end = last_space
        
        chunk = text[start:end].strip()
        if chunk:  # Only add non-empty chunks
            chunks.append(chunk)
        
        # Move start position, accounting for overlap
        start = end - overlap
        if start >= len(text):
            break
    
    return chunks 