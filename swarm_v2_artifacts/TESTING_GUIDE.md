# Sidebar Component Test Suite

## Overview
This comprehensive test suite validates the `Sidebar` component across multiple dimensions:
- **Structural Integrity**: Verifies correct rendering of all navigation items
- **Navigation Behavior**: Tests route transitions and URL updates
- **Active State Styling**: Ensures correct active link highlighting
- **Data Integrity**: Validates configuration data consistency
- **Accessibility**: Checks ARIA roles and keyboard navigation
- **Edge Cases**: Handles unknown routes and route changes
- **Performance**: Ensures rendering efficiency

## Test Categories

| Category | Test Count | Coverage Focus |
|----------|------------|----------------|
| Structural | 5 | Rendering, order, paths, icons |
| Navigation | 5 | Route transitions |
| Active State | 5 | Active link styling |
| Data Integrity | 4 | Configuration validation |
| Accessibility | 3 | ARIA and keyboard support |
| Edge Cases | 3 | Error handling |
| Performance | 3 | Rendering efficiency |

## Running Tests

### JavaScript Tests (Primary)
```bash
# Install dependencies
npm install

# Run all tests with coverage
npm test

# Run in watch mode during development
npm run test:watch

# Run in CI mode
npm run test:ci
```

### Python Pytest Integration
```bash
# Run all tests through pytest (includes JS tests)
pytest

# Run only JS integration tests
pytest tests/test_sidebar_integration.py -v
```

## Coverage Requirements
- **Branches**: ≥ 90%
- **Functions**: ≥ 95%
- **Lines**: ≥ 95%
- **Statements**: ≥ 95%

## Test File Structure
```
tests/
├── Sidebar.test.js          # Main test suite
├── setupTests.js            # Test setup and cleanup
├── test_sidebar_integration.py  # Pytest bridge
├── __mocks__/
│   ├── fileMock.js          # Static asset mock
│   └── styleMock.js         # CSS module mock
```

## CI/CD Integration
The test suite is designed for CI/CD pipelines:
1. Install Node.js dependencies
2. Run `npm run test:ci` for coverage-gated tests
3. Run `pytest` for unified test reporting
4. Fail build if coverage thresholds are not met

## Troubleshooting

### Common Issues
1. **Missing dependencies**: Run `npm install` before testing
2. **Coverage failures**: Check `jest.config.js` thresholds
3. **React Router errors**: Ensure `MemoryRouter` is used in tests

### Debug Commands
```bash
# Run specific test file
npx jest tests/Sidebar.test.js

# Run test with verbose output
npx jest --verbose

# Run with coverage report
npx jest --coverage --coverageReporters=html
```