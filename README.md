# Actix Web Tutorial - Chapter Guide

This repository contains markdown chapters for a comprehensive YouTube tutorial on actix-web, a powerful and fast web framework for Rust.

## Tutorial Structure

The tutorial is designed for developers who already know Rust basics and want to learn how to build web applications with actix-web version 4.12.0.

### Chapter Overview

1. **01-basic-setup.md** - Basic Setup
   - Creating a new project
   - Adding dependencies
   - Your first "Hello World" server
   - Understanding the basic structure

2. **02-routing.md** - Routing
   - HTTP methods (GET, POST, PUT, DELETE)
   - Path parameters
   - Query parameters
   - Request bodies (JSON)
   - Complete routing examples

3. **03-extractors-state.md** - Extractors & Application State
   - Understanding extractors (Path, Query, Json, Form)
   - HttpRequest extractor
   - String and Bytes extractors
   - Application state with web::Data
   - Shared mutable state with Arc
   - Custom extractors
   - Multiple extractors in handlers

4. **04-static-content-htmx.md** - Static Content & HTMX
   - Serving static files
   - HTML templates
   - HTMX integration
   - Form handling with HTMX
   - Server-sent events

5. **05-authentication-github.md** - Authentication (GitHub OAuth)
   - GitHub OAuth setup
   - Session management
   - Protected routes
   - User authentication flow

6. **06-middleware.md** - Middleware
   - Built-in middleware (logging, CORS)
   - Custom middleware
   - Request timing
   - Authentication middleware
   - Middleware ordering

7. **07-database-sqlite.md** - Database (SQLite)
   - Connection pooling
   - Migrations
   - CRUD operations
   - Transactions
   - Error handling

8. **08-error-handling-json.md** - Error Handling & JSON
   - Custom error types
   - Error responses
   - JSON serialization
   - Validation
   - Best practices

9. **09-testing-best-practices.md** - Testing & Best Practices
   - Unit tests
   - Integration tests
   - Testing with databases
   - Production checklist
   - Configuration management

## How to Use These Chapters

Each chapter is a self-contained markdown file with:
- Clear explanations of concepts
- Ready-to-paste code snippets
- Step-by-step instructions
- Key concepts summaries
- Best practices

## Prerequisites

- Basic knowledge of Rust
- Rust toolchain installed (rustc, cargo)
- Text editor or IDE
- Terminal/command line access

## Getting Started

1. Create a new Rust project:
   ```bash
   cargo new actix-tutorial
   cd actix-tutorial
   ```

2. Start with Chapter 1 and follow along

3. Build incrementally, adding features as you progress

## Additional Resources

- [Actix Web Documentation](https://actix.rs/)
- [Actix Web Examples](https://github.com/actix/examples)
- [Rust Book](https://doc.rust-lang.org/book/)
- [HTMX Documentation](https://htmx.org/)

## Tips for Success

1. **Type the code**: Don't just copy-paste - typing helps you learn
2. **Experiment**: Modify examples to see what happens
3. **Read errors**: Rust's error messages are helpful
4. **Test everything**: Run your code frequently
5. **Ask questions**: Use the Rust community for help

## Project Structure

As you progress through the chapters, your project structure will evolve:

```
actix-tutorial/
├── src/
│   └── main.rs
├── static/
│   ├── css/
├── templates/
├── migrations/
├── Cargo.toml
└── .env (for secrets)
```

## Common Issues

### Port Already in Use
If port 8080 is in use, change it in your code or kill the process:
```bash
lsof -ti:8080 | xargs kill
```

### Database Locked
SQLite can have locking issues. Make sure to:
- Close connections properly
- Use connection pooling
- Don't share connections across threads incorrectly

### Compilation Errors
- Check that all dependencies are in Cargo.toml
- Run `cargo clean` if you have weird errors

## Next Steps After Tutorial

Once you've completed all chapters, you'll be ready to:
- Build your own web applications
- Explore advanced features

Good luck with your actix-web journey!s
