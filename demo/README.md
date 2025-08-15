# TLDR Demo Project

This is a demo project to showcase TLDR's semantic search capabilities.

## Features

- **Authentication System**: User login and registration
- **Database Management**: SQLite database operations
- **API Endpoints**: RESTful API with JSON responses
- **Configuration**: YAML-based configuration management

## Quick Start

1. Install dependencies
2. Configure the database
3. Start the server
4. Test the API endpoints

## Architecture

The project follows a clean architecture pattern with:

- Controllers for handling HTTP requests
- Services for business logic
- Repositories for data access
- Models for data structures

## API Documentation

### Authentication Endpoints

- `POST /auth/login` - User login
- `POST /auth/register` - User registration
- `GET /auth/profile` - Get user profile

### User Management

- `GET /users` - List all users
- `GET /users/{id}` - Get user by ID
- `PUT /users/{id}` - Update user
- `DELETE /users/{id}` - Delete user

## Configuration

The application uses YAML configuration files for:

- Database settings
- API configuration
- Logging settings
- Security settings
