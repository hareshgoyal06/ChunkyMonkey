#!/usr/bin/env python3
"""
Test script for ChunkyMonkey's fortified RAG pipeline
This script demonstrates the various RAG capabilities and fallback strategies
"""

import subprocess
import sys
import time

def run_command(cmd, description):
    """Run a command and display the result"""
    print(f"\n{'='*60}")
    print(f"🧪 Testing: {description}")
    print(f"{'='*60}")
    print(f"Command: {cmd}")
    print("-" * 60)
    
    try:
        result = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=30)
        if result.stdout:
            print("✅ Output:")
            print(result.stdout)
        if result.stderr:
            print("⚠️  Errors/Warnings:")
            print(result.stderr)
        if result.returncode != 0:
            print(f"❌ Command failed with return code: {result.returncode}")
        return result.returncode == 0
    except subprocess.TimeoutExpired:
        print("⏰ Command timed out after 30 seconds")
        return False
    except Exception as e:
        print(f"💥 Error running command: {e}")
        return False

def test_rag_pipeline():
    """Test the fortified RAG pipeline features"""
    
    print("🐒 Testing ChunkyMonkey's Fortified RAG Pipeline 🍌")
    print("This test demonstrates the various RAG capabilities and fallback strategies")
    
    # Test 1: Basic RAG pipeline statistics
    success1 = run_command("./cm rag-stats", "RAG Pipeline Statistics")
    
    # Test 2: Database statistics
    success2 = run_command("./cm stats", "Database Statistics")
    
    # Test 3: Test with a simple question (if documents are indexed)
    success3 = run_command('./cm ask "What is this project about?"', "Basic RAG Question")
    
    # Test 4: Test with a more complex question
    success4 = run_command('./cm ask "How does the RAG pipeline work?"', "Complex RAG Question")
    
    # Test 5: Test with a technical question
    success5 = run_command('./cm ask "What are the main components of the system?"', "Technical RAG Question")
    
    # Summary
    print(f"\n{'='*60}")
    print("📊 Test Summary")
    print(f"{'='*60}")
    
    tests = [
        ("RAG Pipeline Statistics", success1),
        ("Database Statistics", success2),
        ("Basic RAG Question", success3),
        ("Complex RAG Question", success4),
        ("Technical RAG Question", success5),
    ]
    
    passed = 0
    total = len(tests)
    
    for test_name, success in tests:
        status = "✅ PASS" if success else "❌ FAIL"
        print(f"{test_name}: {status}")
        if success:
            passed += 1
    
    print(f"\nOverall: {passed}/{total} tests passed")
    
    if passed == total:
        print("🎉 All tests passed! The fortified RAG pipeline is working correctly.")
    else:
        print("⚠️  Some tests failed. Check the output above for details.")
        print("💡 Note: Some tests may fail if no documents are indexed yet.")
    
    return passed == total

def main():
    """Main test function"""
    try:
        success = test_rag_pipeline()
        sys.exit(0 if success else 1)
    except KeyboardInterrupt:
        print("\n⏹️  Test interrupted by user")
        sys.exit(1)
    except Exception as e:
        print(f"\n💥 Unexpected error: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
