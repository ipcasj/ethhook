## Description
<!-- Provide a brief description of the changes in this PR -->

## Type of Change
<!-- Mark the relevant option with an "x" -->

- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] New feature (non-breaking change which adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update
- [ ] Performance improvement
- [ ] Refactoring (no functional changes)
- [ ] Test coverage improvement

## Related Issues
<!-- Link to related issues: Fixes #123, Closes #456 -->

## Changes Made
<!-- List the main changes made in this PR -->

- 
- 
- 

## Testing
<!-- Describe the tests you ran and how to reproduce them -->

- [ ] All existing tests pass (`cargo test --workspace`)
- [ ] Added new tests for new functionality
- [ ] Manually tested the changes
- [ ] Integration tests pass (if applicable)

**Test commands:**
```bash
cargo test --workspace
cargo test -p ethhook-admin-api --test integration_test -- --include-ignored
```

## Checklist
<!-- Mark completed items with an "x" -->

- [ ] My code follows the project's style guidelines
- [ ] I have performed a self-review of my code
- [ ] I have commented my code, particularly in hard-to-understand areas
- [ ] I have updated the documentation (if applicable)
- [ ] My changes generate no new warnings
- [ ] I have added tests that prove my fix is effective or that my feature works
- [ ] New and existing unit tests pass locally with my changes
- [ ] Any dependent changes have been merged and published

## Database Changes
<!-- If this PR includes database migrations -->

- [ ] No database changes
- [ ] New migration added: `migrations/XXX_description.sql`
- [ ] SQLx query cache updated (`cargo sqlx prepare --workspace`)

## Performance Impact
<!-- Describe any performance implications -->

- [ ] No performance impact
- [ ] Performance improved
- [ ] Potential performance impact (explain below)

## Security Considerations
<!-- Describe any security implications -->

- [ ] No security implications
- [ ] Security improvements included
- [ ] Potential security concerns (explain below)

## Screenshots / Logs
<!-- Add screenshots or relevant logs if applicable -->

## Additional Context
<!-- Add any other context about the PR here -->
