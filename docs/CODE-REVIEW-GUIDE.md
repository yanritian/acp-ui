# Code Review Guide

This guide provides standards and best practices for reviewing code in Hermes Game Operator.

## Review Philosophy

### Goals

1. **Improve Code Quality** - Catch bugs, security issues, and design problems
2. **Share Knowledge** - Learn from each other's approaches
3. **Maintain Standards** - Ensure consistency across the codebase
4. **Mentor Contributors** - Help others grow as developers

### Principles

- **Be Constructive** - Focus on improvement, not criticism
- **Be Specific** - Provide clear, actionable feedback
- **Be Respectful** - Treat others with dignity and respect
- **Be Timely** - Review promptly to unblock progress

## Review Checklist

### Code Quality

- [ ] Code follows project coding standards
- [ ] Functions are small and focused (< 50 lines)
- [ ] Files are organized logically (< 800 lines)
- [ ] No code duplication
- [ ] Proper error handling
- [ ] No hardcoded values (use constants)
- [ ] Meaningful variable and function names
- [ ] Comments explain "why", not "what"

### TypeScript/Vue

- [ ] Proper type annotations
- [ ] No `any` types unless absolutely necessary
- [ ] Vue 3 Composition API used correctly
- [ ] Props and emits properly typed
- [ ] Reactive state used appropriately
- [ ] Computed properties for derived state
- [ ] Proper lifecycle hooks usage
- [ ] No memory leaks (event listeners cleaned up)

### Rust

- [ ] Proper error handling (Result types)
- [ ] No unwrap() in production code
- [ ] Proper ownership and borrowing
- [ ] No unsafe code unless necessary
- [ ] Proper documentation comments
- [ ] Tests for public functions
- [ ] Proper use of async/await
- [ ] Thread safety considerations

### Security

- [ ] No hardcoded secrets
- [ ] Input validation present
- [ ] SQL injection prevention
- [ ] XSS prevention
- [ ] CSRF protection
- [ ] Proper authentication/authorization
- [ ] Rate limiting on endpoints
- [ ] Error messages don't leak sensitive data

### Performance

- [ ] No unnecessary computations
- [ ] Proper caching where appropriate
- [ ] No memory leaks
- [ ] Efficient algorithms
- [ ] Proper resource cleanup
- [ ] No blocking operations in async code
- [ ] Proper use of indexes (database)
- [ ] Pagination for large datasets

### Testing

- [ ] Unit tests included
- [ ] Tests cover happy path
- [ ] Tests cover edge cases
- [ ] Tests cover error cases
- [ ] Tests are readable and maintainable
- [ ] No flaky tests
- [ ] Test coverage > 80%

### Documentation

- [ ] README updated if needed
- [ ] API documentation updated
- [ ] Inline comments added for complex logic
- [ ] Examples provided for new features
- [ ] Changelog updated

## Review Process

### For Reviewers

#### 1. Understand Context

- Read the PR description
- Check linked issues
- Understand the goal
- Review related code

#### 2. High-Level Review

- Architecture changes
- Design patterns
- API design
- Breaking changes

#### 3. Detailed Review

- Code quality
- Security
- Performance
- Testing
- Documentation

#### 4. Provide Feedback

- **Positive**: Highlight good practices
- **Constructive**: Suggest improvements
- **Questions**: Ask for clarification
- **Concerns**: Raise potential issues

#### 5. Make Decision

- **Approve**: Code is ready to merge
- **Request Changes**: Changes needed before merge
- **Comment**: Just providing feedback

### For Contributors

#### 1. Self-Review

- Review your own code first
- Check against checklist
- Run all tests
- Update documentation

#### 2. Prepare PR

- Clear title and description
- Link related issues
- Provide context
- Add screenshots if UI changes

#### 3. Respond to Feedback

- Acknowledge all comments
- Address concerns
- Ask for clarification if needed
- Make requested changes

#### 4. Update PR

- Push changes
- Re-request review
- Provide update on changes

## Common Issues

### TypeScript/Vue

#### Issue: Complex Type Inference

```typescript
// Problem: Type instantiation is excessively deep
const nodeTypes = { agent: AgentNode }

// Solution: Use explicit type annotation
const nodeTypes: Record<string, any> = { agent: AgentNode }
```

#### Issue: Reactive State Issues

```typescript
// Problem: Not reactive
const count = 0

// Solution: Use ref
const count = ref(0)
```

#### Issue: Memory Leaks

```typescript
// Problem: Event listener not cleaned up
onMounted(() => {
  window.addEventListener('resize', handleResize)
})

// Solution: Clean up in onUnmounted
onMounted(() => {
  window.addEventListener('resize', handleResize)
})

onUnmounted(() => {
  window.removeEventListener('resize', handleResize)
})
```

### Rust

#### Issue: Unwrap in Production

```rust
// Problem: Can panic
let value = map.get("key").unwrap();

// Solution: Proper error handling
let value = map.get("key")
    .ok_or_else(|| Error::NotFound("key".to_string()))?;
```

#### Issue: Ownership Issues

```rust
// Problem: Borrowing conflict
let s1 = String::from("hello");
let s2 = s1; // s1 moved
println!("{}", s1); // Error

// Solution: Clone if needed
let s1 = String::from("hello");
let s2 = s1.clone();
println!("{}", s1); // OK
```

#### Issue: Async/Await Issues

```rust
// Problem: Blocking in async
async fn process() {
    std::thread::sleep(Duration::from_secs(1)); // Blocking!
}

// Solution: Use async sleep
async fn process() {
    tokio::time::sleep(Duration::from_secs(1)).await;
}
```

## Feedback Examples

### Good Feedback

```markdown
**Positive**: Great use of the strategy pattern here! It makes the code
very extensible.

**Constructive**: Consider extracting this logic into a separate function
to improve readability. Something like `validateInput()` would make the
intent clearer.

**Question**: Why did you choose to use a HashMap here instead of a Vec?
I'm curious about the performance implications.

**Concern**: This could be a security issue if the input is not sanitized.
Can we add validation before processing?
```

### Bad Feedback

```markdown
❌ "This is wrong."
✅ "This might cause issues because..."

❌ "Why didn't you do it this way?"
✅ "Have you considered this approach? It might be better because..."

❌ "This needs to be fixed."
✅ "I suggest changing this to... because..."
```

## Review Time Guidelines

- **Small PRs** (< 200 lines): 30 minutes
- **Medium PRs** (200-500 lines): 1 hour
- **Large PRs** (500-1000 lines): 2 hours
- **Very Large PRs** (> 1000 lines): Consider splitting

## Automated Checks

### CI/CD Pipeline

- [ ] All tests pass
- [ ] Type checking passes
- [ ] Linting passes
- [ ] Build succeeds
- [ ] No security vulnerabilities
- [ ] Code coverage maintained

### Pre-commit Hooks

- [ ] Code formatted
- [ ] Imports sorted
- [ ] No console.log statements
- [ ] No TODO comments

## Tools

### Code Quality

- **ESLint**: JavaScript/TypeScript linting
- **Prettier**: Code formatting
- **Clippy**: Rust linting
- **cargo fmt**: Rust formatting

### Security

- **npm audit**: Node.js security
- **cargo audit**: Rust security
- **Snyk**: Dependency scanning
- **CodeQL**: Code scanning

### Testing

- **Vitest**: Unit testing
- **Playwright**: E2E testing
- **cargo test**: Rust testing
- **Coverage reports**

## Resources

- [Clean Code](https://www.amazon.com/Clean-Code-Handbook-Software-Craftsmanship/dp/0132350882)
- [The Pragmatic Programmer](https://www.amazon.com/Pragmatic-Programmer-journey-mastery-Anniversary/dp/0135957052)
- [Code Complete](https://www.amazon.com/Code-Complete-Practical-Handbook-Construction/dp/0735619670)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Vue 3 Guide](https://vuejs.org/guide/)

---

**Happy reviewing!** 🔍

---

**Last Updated**: 2026-07-08  
**Version**: 1.0.0
