"""
Authentication System for Demo Project

This module provides user authentication functionality including
login, registration, and profile management.
"""

import hashlib
import jwt
from datetime import datetime, timedelta
from typing import Optional, Dict, Any

class UserAuth:
    """Handles user authentication and authorization."""
    
    def __init__(self, secret_key: str = "demo-secret-key"):
        self.secret_key = secret_key
        self.users = {}  # In-memory user storage for demo
        
    def register_user(self, username: str, email: str, password: str) -> Dict[str, Any]:
        """
        Register a new user account.
        
        Args:
            username: Unique username
            email: User email address
            password: Plain text password
            
        Returns:
            Dict containing registration status and user info
        """
        if username in self.users:
            return {
                "success": False,
                "error": "Username already exists"
            }
        
        # Hash the password
        hashed_password = self._hash_password(password)
        
        # Create user record
        user = {
            "id": len(self.users) + 1,
            "username": username,
            "email": email,
            "password_hash": hashed_password,
            "created_at": datetime.now(),
            "is_active": True
        }
        
        self.users[username] = user
        
        return {
            "success": True,
            "user": {
                "id": user["id"],
                "username": user["username"],
                "email": user["email"],
                "created_at": user["created_at"]
            }
        }
    
    def login_user(self, username: str, password: str) -> Dict[str, Any]:
        """
        Authenticate a user and return JWT token.
        
        Args:
            username: User's username
            password: Plain text password
            
        Returns:
            Dict containing login status and JWT token
        """
        if username not in self.users:
            return {
                "success": False,
                "error": "Invalid credentials"
            }
        
        user = self.users[username]
        
        if not user["is_active"]:
            return {
                "success": False,
                "error": "Account is deactivated"
            }
        
        if not self._verify_password(password, user["password_hash"]):
            return {
                "success": False,
                "error": "Invalid credentials"
            }
        
        # Generate JWT token
        token = self._generate_token(user)
        
        return {
            "success": True,
            "token": token,
            "user": {
                "id": user["id"],
                "username": user["username"],
                "email": user["email"]
            }
        }
    
    def get_user_profile(self, token: str) -> Optional[Dict[str, Any]]:
        """
        Get user profile from JWT token.
        
        Args:
            token: JWT authentication token
            
        Returns:
            User profile data or None if invalid token
        """
        try:
            payload = jwt.decode(token, self.secret_key, algorithms=["HS256"])
            username = payload.get("username")
            
            if username in self.users:
                user = self.users[username]
                return {
                    "id": user["id"],
                    "username": user["username"],
                    "email": user["email"],
                    "created_at": user["created_at"],
                    "is_active": user["is_active"]
                }
        except jwt.InvalidTokenError:
            pass
        
        return None
    
    def _hash_password(self, password: str) -> str:
        """Hash password using SHA-256."""
        return hashlib.sha256(password.encode()).hexdigest()
    
    def _verify_password(self, password: str, hashed: str) -> bool:
        """Verify password against hash."""
        return self._hash_password(password) == hashed
    
    def _generate_token(self, user: Dict[str, Any]) -> str:
        """Generate JWT token for user."""
        payload = {
            "user_id": user["id"],
            "username": user["username"],
            "exp": datetime.utcnow() + timedelta(hours=24)
        }
        return jwt.encode(payload, self.secret_key, algorithm="HS256")

# Example usage
if __name__ == "__main__":
    auth = UserAuth()
    
    # Register a user
    result = auth.register_user("demo_user", "demo@example.com", "password123")
    print("Registration:", result)
    
    # Login
    login_result = auth.login_user("demo_user", "password123")
    print("Login:", login_result)
    
    if login_result["success"]:
        # Get profile
        profile = auth.get_user_profile(login_result["token"])
        print("Profile:", profile) 