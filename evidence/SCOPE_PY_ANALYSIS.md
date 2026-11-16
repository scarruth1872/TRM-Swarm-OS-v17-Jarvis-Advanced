# CPython `test_scope.py` — GOTCHA ANALYSIS

**File analysed:** `C:/Users/shawn/AppData/Local/Programs/Python/Python311/Lib/test/test_scope.py` (815 lines, 40 test methods)
**Verified against:** CPython **3.11.9** (owner of the file) and **3.14.9** (installed default). All 40 tests pass unchanged on both — the file is behaviour-stable, which is itself informative (§4.4).

Method: every test read; the load-bearing claims re-derived by executing probes and reading `dis` output rather than from memory. Probe transcripts are quoted inline under "evidence".

---

## 1. THE SCOPE MODEL THE TESTS ASSUME

**LEGB, with the class body as a one-way trapdoor.**

* Names resolve Local → Enclosing function scopes → Global → Builtins.
* A function body is a real lexical scope: a nested `def` gets *cells* for names it reads from enclosing function scopes (`testSimpleNesting`, `testExtraNesting`).
* **A class body is a scope for the code written directly inside it, but NOT an enclosing scope for functions defined inside it.** This is the single most load-bearing asymmetry in the file. Proven by opcode choice:

```
class C:
    cx = 2
    val = cx            # LOAD_NAME  cx   <- class body checks its own namespace first
    def m(self):
        return cx       # LOAD_GLOBAL cx  <- method never sees the class namespace
```

  `testFreeVarInMethod`, `testBoundAndFree`, `testLocalsClass` all exist to pin this.
* Class body name lookup uses **`LOAD_NAME`**, i.e. class namespace → module globals → builtins, and it *is* mutable at runtime through `locals()` (`testClassNamespaceOverridesClosure`).
* Function bodies use `LOAD_FAST`/`LOAD_DEREF`/`LOAD_GLOBAL` — never `LOAD_NAME` — so no runtime dict can inject a name into a running function.

**`global` / `nonlocal` applicability**

| Statement | Function body | Class body | Module |
|---|---|---|---|
| `global x` | legal; binds all `x` in that *block* to module | legal; affects the class-body block only | no-op |
| `nonlocal x` | legal if a binding exists in an enclosing **function** | legal if the class body is nested in a function | `SyntaxError: no binding for nonlocal 'x' found` |

`nonlocal` needs a binding in an enclosing *function* scope — never a class body, never the module. Verified:

```
nonlocal-in-class-body: (1, 1)              # works, x is the enclosing function's cell
nonlocal-at-module-class: SyntaxError: no binding for nonlocal 'q' found
nonlocal->global:         SyntaxError: no binding for nonlocal 'x' found
```

**Comprehensions.** Comprehensions are their own function scope; the loop variable does not leak (`testListCompLocalVars`). The *outermost iterable* is evaluated in the enclosing scope, everything else inside. This distinction is not tested here (§4.1) but is implied by the model and confirmed by probe.

**Closures / cells.** One cell per binding, shared by every closure created in that scope. Rebinding the name in the defining scope is visible to already-created closures (`testSimpleAndRebinding`). Free variables pass through intermediate scopes that never use them (`testExtraNesting`). A cell handed in as an *argument* is re-wrapped in a fresh cell, never aliased (`testCellIsArgAndEscapes`).

**Module vs function namespace.** Module-level code is `LOAD_NAME`/`STORE_NAME` against a dict; function code is index-based against a frame array. That difference is exactly what `exec()` with separate globals/locals dicts exposes (§3), and what `testGlobalInParallelNestedFunctions` accidentally depends on.

---

## 2. GOTCHA CATALOGUE

### A. Class body scoping

**1. A class attribute is invisible to the methods defined in the same class.**
`testBoundAndFree` / `testLocalsClass`
```python
def f(x):
    class C:
        x = 12
        def m(self):
            return x
    return C
# C.x == 12, but C().m() == 1  (the enclosing function's x, via closure)
```
`m` compiles to `LOAD_GLOBAL`, so `C.x` is not even a candidate. If you expect a class attribute to act as a default for methods, you get either the function's free variable or a `NameError` — never the class attribute. (Verified: `C.x = 12 | C.m() = 1`.)

**2. `locals()` in a class body is live and outranks the enclosing scope.**
`testClassNamespaceOverridesClosure` (#17853)
```python
x = 42
class X:
    locals()["x"] = 43
    y = x
self.assertEqual(X.y, 43)
```
Writing into the class namespace changes what subsequent class-body expressions resolve to — the global `x = 42` is never consulted. This works *only* because class bodies use `LOAD_NAME`. Everywhere else in Python this trick is a no-op.

**3. `del x` in a class body deletes the class-namespace entry, not the enclosing/global one.**
`testClassNamespaceOverridesClosure`
```python
class X:
    locals()["x"] = 43
    del x
self.assertFalse(hasattr(X, "x"))
self.assertEqual(x, 42)          # global untouched
```
`del` follows the same `LOAD_NAME` visibility rules as reads. If you implement class scopes as "just another dict layer", you will delete the wrong binding.

**4. `global` inside a class body makes the class-body name a module store, but does not make the method's `x` follow the method's own assignment.**
`testScopeOfGlobalStmt` (the `Global` class, lines 442–454)
```python
x = 12
class Global:
    global x
    x = 13
    def set(self, val):
        x = val
    def get(self):
        return x
g = Global(); g.get()  # 13
g.set(15);   g.get()  # still 13
```
`set()` assigns a *local* `x` that dies with the call. `get()` reads the module global (13). The class body's `global` declaration has zero effect on either method. Verified: `get before=13, after set=13, module x=13`.

**5. A nested `def` inside a class body cannot see the class namespace, but the class body can see the enclosing function's locals.**
`testFreeVarInMethod`
```python
def test():
    method_and_var = "var"
    class Test:
        def method_and_var(self): return "method"
        def test(self):           return method_and_var   # free var -> "var"
        def actual_global(self):  return str("global")    # NOT self.str
        def str(self):            return str(self)        # recurses into global str
```
`actual_global()` returns `"global"`: the sibling method `str` is not in scope for it. The method `str` calls the *builtin*, because `LOAD_GLOBAL` finds `str` in builtins. If you assume "methods share the class namespace", this whole test reads as nonsense.

**6. The same class nested vs. not nested behaves identically here — the comment says otherwise.**
`testFreeVarInMethod`, lines 152–167: `# this class is not nested, so the rules are different` — yet every assertion is the same (`"var"`, `"method"`, `"global"`). The only real difference is that in the nested case `method_and_var` is a *cell*, and in the flat case it is a module global. Both are invisible to methods, so the assertion cannot detect the difference. Do not cite this half as evidence of a rule.

**7. `nonlocal` is legal in a class body and must not create a class attribute.**
`testNonLocalClass`
```python
def f(x):
    class c:
        nonlocal x
        x += 1
        def get(self):
            return x
    return c()
c.get() == 1
self.assertNotIn("x", c.__class__.__dict__)
```
`x += 1` at class-body level loads and stores the *enclosing function's cell*. The natural (wrong) implementation — treat the class body as a function with `x` as a fast local — would put `x` in the class dict and leave the closure at 0. Verified: `nonlocal-in-class-body: 1, class dict has x: False`.

**8. Calling `locals()` / installing a trace function in a class body must not copy free variables into the class namespace.**
`testLocalsClass` (#23728 family), `testLocalsClass_WithTrace`
```python
def f(x):
    class C:
        y = x
        def m(self): return x
        z = list(locals())
self.assertNotIn("x", f(1).z)      # verified: ['__module__','__qualname__','y','m']
self.assertIn("y", f(1).z)
```
The frame synchronisation code that round-trips fast locals through `f_locals` (trace functions, `locals()`) used to include free/cell variables. In a class body that meant injecting `x` into the class dict, which then made `x = 12` in the class body look like a local rebinding — and the traced variant used to raise `UnboundLocalError` on the *global* `x` afterwards:
```python
sys.settrace(lambda a,b,c:None)
x = 12
class C:
    def f(self): return x
self.assertEqual(x, 12)   # Used to raise UnboundLocalError
```
Both tests are `@cpython_only` — this is frame-internals behaviour, not language semantics.

### B. `global` / `nonlocal` edge cases

**9. `global x` in an inner function makes the innermost `x` invisible to everything below it, skipping the enclosing function's local entirely.**
`testScopeOfGlobalStmt`, case I vs II (lines 376–405)
```python
x = 7
def f():
    x = 1                 # f's local — must be SKIPPED
    def g():
        global x
        def i():
            def h(): return x
```
Case I returns `7` (module), case II (no `global`) returns `2` (innermost enclosing). The `global` in `g` re-points `h`'s cell lookup to the module, so `f`'s `x = 1` becomes dead weight. Implementations that resolve "nearest enclosing binding" without honouring a `global` marker in an intermediate scope return `1` here.

**10. A `global` declaration in one function must not leak to a sibling function — but it *does* upgrade the module block's store to `STORE_GLOBAL`.**
`testGlobalInParallelNestedFunctions` (#9997 family)
```python
def f():
    y = 1
    def g():
        global y
        return y          # -> module y = 9
    def h():
        return y + 1      # -> f's local y = 1, i.e. 2
    return g, h
y = 9
result9 = g()   # 9
result2 = h()   # 2
```
Two things surprise people. First, `g()` and `h()` read *different* `y`s from textually adjacent code. Second, the test reads the results out of `global_ns`, and that is not incidental — the nested `global y` makes the **module block** emit `STORE_GLOBAL y`, so `y = 9` bypasses the `locals` dict handed to `exec`:

```
A plain module exec      STOREs: ['STORE_NAME:y', 'STORE_NAME:zz']       globals: []         locals: ['y','zz']
B with nested 'global y' STOREs: ['STORE_NAME:f','STORE_GLOBAL:y', ...]  globals: ['y']      locals: ['f','zz']
```
This is the residue of the symbol-table leak the test guards: the declaration is scoped to `g`, but its *effect on the module block's binding mode* is real and observable.

**11. `global a` where a sibling function has a parameter named `a` must not be a syntax error.**
`testTopIsNotSignificant` (#9997)
```python
def top(a):
    pass
def b():
    global a
```
Nothing is asserted beyond "this compiles and runs". The failure mode it prevents is a symbol-table leak that produces `SyntaxError: name 'a' is parameter and global` on `top` — verified that this is the exact message when the declaration genuinely applies to `top`. A regression here surfaces as a hard compile error in unrelated code, which is why the test needs no assertions.

**12. `nonlocal` chains transitively through intermediate functions, even ones that also declare `nonlocal`.**
`testNestedNonLocal`
```python
def f(x):
    def g():
        nonlocal x
        x -= 2
        def h():
            nonlocal x        # binds THROUGH g to f's cell
            x += 4
```
`f(1)` → `g()` → `h()` returns `3` (1 − 2 + 4). `g` creating a cell for `x` does not stop `h` reaching `f`'s cell; the intermediate `nonlocal` only means "don't create a new binding here".

**13. Methods and generators get ordinary function-scope cells; `nonlocal` works identically in both.**
`testNonLocalMethod`, `testNonLocalGenerator`
```python
class c:
    def inc(self):
        nonlocal x
        x += 1
```
and
```python
def g(y):
    nonlocal x
    for i in range(y):
        x += 1
        yield x
```
A generator's frame keeps the shared cell alive across suspension — `list(g(5)) == [1,2,3,4,5]` accumulates on the same `x` as the enclosing function. Treating generator bodies as a separate namespace (or snapshotting cells at generator creation) breaks this.

**14. `x += 1` on a module global without `global` is `UnboundLocalError`, not a global update.**
`testUnboundLocal_AugAssign` (#1501934)
```python
global_x = 1
def f():
    global_x += 1     # LOAD_FAST/STORE_FAST, because += implies a binding
f()                   # UnboundLocalError
```
A compound assignment marks the name local *before* the load. The test runs this inside `exec(...)` with `{'fail': self.fail}` injected precisely because historically the compiler emitted `LOAD_GLOBAL`/`STORE_FAST` (or the reverse) — a mismatched pair that silently corrupted the global instead of raising.

**15. A `global` statement affects its whole block regardless of position.**
`testNestingPlusFreeRefToGlobal`
```python
def make_adder6(x):
    global global_nest_x
    def adder(y):
        return global_nest_x + y
    global_nest_x = x
```
Two callers share one global: `inc(1) == 11` while `plus10(-2) == 8`. The test comment says it outright — `# there's only one global`. Creating a fresh module-level name per closure is the bug this catches; the second `assertEqual` is a time-bomb for anyone who caches globals at definition time.

### C. Comprehension scoping

**16. Comprehension loop variables never leak — this held before and after the 3.12 inlining change.**
`testListCompLocalVars`
```python
def x():
    [bad for s in 'a b' for bad in s.split()]
x()
# NameError: name 'bad' is not defined
```
Verified on 3.11 (separate function scope) and 3.14 (PEP 709 inlined) — same observable result. The `try/except NameError` around a bare `print(bad)` at the top of the test is asserting the *global* namespace is clean before and after, so a leak into `globals()` is caught, not just a leak into `locals()`.

### D. Lambdas and closures

**17. Rebinding a captured name after the closure is created changes what the closure sees.**
`testSimpleAndRebinding`
```python
def make_adder3(x):
    def adder(y): return x + y
    x = x + 1        # after adder was made
    return adder
inc = make_adder3(0); inc(1) == 2
```
The cell, not the value, is captured. A compiler that snapshots free variables at `MAKE_FUNCTION` time passes `testSimpleNesting` and fails this one.

**18. Free variables traverse intermediate scopes that never mention them.**
`testExtraNesting` — `# check freevars passing through non-use scopes`. `x` is used only in `adder`, two `def`s deep. Any "capture only what the immediately enclosing function references" scheme breaks.

**19. The nearest enclosing binding wins; an inner rebinding masks an outer one.**
`testNearestEnclosingScope`
```python
def f(x):          # 10
    def g(y):
        x = 42     # masks f's x for h
        def h(z): return x + z
```
`f(10)` then `(5)` returns `47`, not `15`. The `# check that this masks binding in f()` comment is the whole test.

**20. A name can be a cellvar in one function and a freevar in its child, and assignment order is irrelevant to correctness.**
`testMixedFreevarsAndCellvars` — `y` is assigned (`y = c + z`) *after* `h` (which reads `y`) is defined; `b` is a parameter of `g` read by `h`; `identity` is a global call. Expected `39`. This is the frame-array layout test: `freevars`, `cellvars`, `varnames` are three separate index spaces and the compiler must not confuse them.

**21. A keyword-only parameter with a default becomes a cell correctly (Issue 1409).**
`testCellIsKwonlyArg`
```python
def foo(*, a=17):
    def bar(): return a + 5
    return bar() + 3
foo(a=42) == 50 ; foo() == 25
```
Self-referential gotcha: fixing this bug by treating "has default" as "not a cell" makes `bar` read a stale `None` cell. Both the supplied and defaulted paths are asserted.

**22. A cell passed in as an argument is *not* reused as the closure cell.**
`testCellIsArgAndEscapes`, `testCellIsLocalAndEscapes`
```python
cell_ext, = external().__closure__
eggs = spam(cell_ext)
cell_closure, = eggs.__closure__
self.assertIs(cell_eggs, cell_ext)        # contents: the cell object itself
self.assertIsNot(cell_eggs, cell_closure) # but NOT the same cell object
```
`spam(arg)` must allocate a new cell wrapping `arg`; "optimising" by reusing the incoming cell would alias two logically distinct variables (verified: `returned is outer cell: True`, `closure cell is outer cell: False`).

**23. Closures must not keep objects alive beyond the frame, but a frame/cell cycle must still be collectable.**
`testLeaks` — 100 iterations of a closure over a `Foo()` instance must end with `Foo.count == 0` after `gc_collect()`.
`testCellLeak` (#17927) — `self` forced into a cell by `lambda: self`, plus an exception stored on `self.exc`, creates a frame↔instance cycle; `self = None` alone used to be insufficient because the frame held a second reference:
```python
def dig(self):
    if 0:
        lambda: self
    ...
    self = None  # Break the cycle
```
`if 0: lambda: self` is the idiom for "force `self` into a cell without executing anything" — worth knowing when writing your own repro.

**24. Recursion through a closure works because the function's own name is a cell in the defining scope.**
`testRecursion` — `fact` calls itself inside `f`. If your implementation makes `def` bind only after the body is built (or treats the name as a global), recursion raises `NameError`.

### E. Unbound locals and `del`

**25. The *same* textual bug raises two different exception types depending on which frame reads the name.**
`testUnboundLocal` (and identically `testUnboundLocal_AfterDel`)
```python
def errorInOuter():
    print(y)          # UnboundLocalError: cannot access local variable 'y' ...
    def inner(): return y
    y = 1

def errorInInner():
    def inner(): return y
    inner()           # NameError: cannot access free variable 'y' ... in enclosing scope
    y = 1
```
`UnboundLocalError` when the *local* slot is empty; plain `NameError` when a *cell* is empty. Both messages name the variable and the reason (verified on 3.11 and 3.14). `UnboundLocalError` is a subclass of `NameError`, so `except NameError` catches both — which is why the test uses `assertRaises` on the distinct types rather than on the base class.

**26. Deleting a cell variable is legal and leaves the cell *empty* rather than unbound/undefined.**
`testUnboundLocal_AfterDel` (#4617) — `# It is now legal to delete a cell variable.`
```python
def errorInOuter():
    y = 1
    del y
    print(y)          # UnboundLocalError
    def inner(): return y
```
Before #4617 this was a compile-time error. After the `del`, every reader — the frame itself and any closure — sees the same "empty cell" state and raises. A closure created *after* the `del` still captures the cell (it exists), so it raises at call time, not at definition time (verified: `NameError: cannot access free variable 'y' ...`).

### F. Star-import and namespace ambiguity

**27. `from x import *` is a `SyntaxError` in function *and* class bodies — the ban is about unresolvable name bindings, not about functions specifically.**
`testUnoptimizedNamespaces` — four variants, each wrapped in `if 1:`:
```python
def f(s):
    from sys import *
    return getrefcount(s)   # ambiguity: free or local
```
Verified: all raise `import * only allowed at module level`, including inside a class body. The inline comments describe *why* (a star-import would make every later name's binding undecidable at compile time), but the enforced rule is simply positional. The `if 1:` wrappers and `check_syntax_error` are vestiges of the era when these snippets were `exec`'d as strings; they no longer change anything.

---

## 3. EXEC AND EVAL SPECIFICS

### 3.1 `exec` at module level with separate globals/locals

`exec(src, g, l)` compiles the source in `exec` mode: `STORE_NAME` targets the **locals** dict. But a `global` declaration anywhere in a *nested* function flips the corresponding module-level store to `STORE_GLOBAL`, which targets the **globals** dict:

```
src = "y = 9\nzz = 1\n"
  STOREs: STORE_NAME:y, STORE_NAME:zz        -> globals: []      locals: ['y','zz']

src = "def f():\n    y = 1\n    def g():\n        global y\n        return y\n    return g\ny = 9\nzz = 1\n"
  STOREs: STORE_NAME:f, STORE_GLOBAL:y, STORE_NAME:zz
                                              -> globals: ['y']   locals: ['f','zz']
```

Consequences: with two dicts, **you cannot predict which dict receives a name without checking for `global` declarations in nested scopes.** `testGlobalInParallelNestedFunctions` (lines 700–717) reads `result9`/`result2` out of `global_ns` and works only because of this; the def-statements and their results land in `local_ns` (`f`, `g`, `h`, `result9`, `result2`), while the `global`-declared `y` lands in `global_ns`. If you pass one dict (`exec(src, ns)`) both go to `ns` and the distinction disappears — which is why the test deliberately passes two.

### 3.2 `exec` inside a function

Three separate facts, all verified:

1. **Reads work.** `exec("b = a + 1")` sees the caller's local `a` (returns 8). `exec` with no explicit namespaces gets a snapshot/mapping of the calling frame's globals and locals.
2. **Writes never bind fast locals.** After `exec("q1 = 1")`, a later `return q1` raises `NameError` on **both 3.11 and 3.14**. The compiler decided `q1` is a global (it was never assigned by any statement in the function body), so it compiles to `LOAD_GLOBAL` and the exec-time dict entry is never consulted.
3. **Where the leftover write goes is version-dependent** — see 3.5.

The corollary for real code: mutating an *existing* local via `exec` is silently discarded too —
```python
def p2():
    v = 1
    exec("v = 99")
    return v          # 1, on 3.11 and 3.14
```
`v` is a fast local; `exec`'s `STORE_NAME` writes the separate locals mapping, which is never copied back. No error, no effect.

### 3.3 `eval` and free variables

`eval("x + 1")` inside a closure sees `x` **only if the enclosing function itself references `x`**:

```python
def f(x):
    def with_ref():
        x                    # <-- this bare expression is load-bearing
        return eval("x + 1")
    def without_ref():
        return eval("x + 1")

with_ref()     -> 5
without_ref()  -> NameError: name 'x' is not defined       # 3.11 and 3.14
```

`testEvalFreeVars` (lines 641–649) contains exactly this stray `x` statement before the `eval`, and it looks like leftover debris — it is not. `eval` sees the calling frame's `f_locals`, and a free variable only appears there if the *compiler* made it a freevar of that function, which requires an actual reference in the body. Delete the bare `x` and the test fails. This is the most consequential gotcha in the file for anyone writing `eval`-based tooling.

### 3.4 Code objects with free variables cannot be `exec`'d or `eval`'d

`testEvalExecFreeVars`
```python
def f(x):
    return lambda: x + 1
g = f(3)
self.assertRaises(TypeError, eval, g.__code__)
exec(g.__code__, {})   # must also raise TypeError
```
Distinct messages, both `TypeError`:
```
eval(code_with_freevars): TypeError: code object passed to eval() may not contain free variables
exec(code_with_freevars, {}): TypeError: code object requires a closure of exactly length 1
```
A code object's `co_freevars` must be satisfied by a closure tuple, which only `types.FunctionType(code, globals, name, argdefs, closure)` can supply. There is no way to bind free variables through `exec`'s namespaces — `testEvalExecFreeVars` is asserting that door is closed (the `try/except/else: self.fail(...)` form is used so that *any* exception other than `TypeError`, including no exception, fails the test).

### 3.5 `locals()` — snapshot vs. write-through, and the 3.11 → 3.13 change

`locals()` returns a fresh dict each call, but on ≤3.12 it is *write-through-ish*: the dict is the frame's `f_locals`, and the frame re-syncs it. Verified divergence:

```python
def b2():
    exec("q2 = 2")
    return locals().get('q2', '<absent>')

3.11  -> 2          # exec's write landed in f_locals, visible to locals()
3.14  -> <absent>   # PEP 667: locals() is an independent snapshot
```

This is **PEP 667** (3.13): `locals()` in an optimized scope returns a detached snapshot; writes to it do not affect fast locals, and `exec`'s implicit namespace is no longer the same object. Meanwhile `locals()` still *reads* fast locals and free variables, and the snapshot is still re-taken on each call:

```python
def snap():
    a = 1
    d = locals(); a = 2
    return d['a'], locals()['a']
-> (1, 2)            # on 3.11 AND 3.14
```
(That "re-taken each call" property is what `testLocalsFunction` depends on: `y += 3` happens before `return locals()`, so `d == {'x': 2, 'y': 7, 'w': 6}` — `x` present as a free variable, `y` and `w` as fast locals, and `h` as the nested function. Verified identical on both versions.)

At **module level** `locals()` *is* `globals()` and remains live — hence `class X: locals()["x"] = 43` works on every version. The write-through rules that apply to class bodies and modules do not apply inside functions.

### 3.6 Trace functions and frame synchronisation (`@cpython_only`)

`testLocalsClass_WithTrace` (#23728) and `testInteractionWithTraceFunc` exist because a trace function forces a full fast-locals ↔ `f_locals` round-trip on every line. Historically that round-trip copied free variables into the class namespace, after which a subsequent class-body assignment to the same name looked like a local rebinding, and the *enclosing* `x` became unbound (`# Used to raise UnboundLocalError`). `testInteractionWithTraceFunc` is a smoke test for that machinery (a tracer that returns itself, an `AV` (attribute-value) path taken while stepping), and its only assertion — `self.assertRaises(TypeError, sys.settrace)` — merely checks that `settrace` still requires an argument. Note both tests are `@cpython_only`: they pin interpreter internals, not language semantics, so a non-CPython implementation can legitimately skip them.

### 3.7 Net version-variance summary

| Behaviour | 3.11 | 3.13+ (verified 3.14.9) |
|---|---|---|
| `exec("q=2")` then `locals().get('q')` in a function | `2` | absent (PEP 667) |
| `exec("q=1")` then `return q` in a function | `NameError` | `NameError` (unchanged) |
| `locals()['a']=99` then `return a` | unchanged | unchanged |
| Comprehension loop var leak | none | none (PEP 709 inlined) |
| `eval` of code with freevars | `TypeError` | `TypeError` |
| `nonlocal` in class body | works | works |

The whole file passes on both versions, which means **no test in it pins the 3.11 write-through `f_locals` behaviour that PEP 667 removed** (§4.4).

---

## 4. WHAT THE TESTS DELIBERATELY DO NOT COVER

**4.1 Comprehensions inside class bodies.**
No test touches this, and it is the most common real-world manifestation of the class-body rule. Verified on 3.11 and 3.14:
```python
class K:
    vals = [1,2,3]
    doubled = [v*2 for v in vals]           # OK -> [2,4,6]
class K:
    vals = [1,2,3]
    doubled = [v*vals[0] for v in vals]     # NameError: name 'vals' is not defined
```
Only the **outermost iterable** is evaluated in the class scope; every other reference to a class name inside the comprehension fails, because the comprehension body is a nested function scope. Same for methods: `return [i*k for i in range(2)]` with `k` a class attribute raises `NameError`. And the walrus is banned outright in this position: `SyntaxError: assignment expression within a comprehension cannot be used in a class body`. `testListCompLocalVars` only checks leakage, so the whole interaction is untested.

**4.2 Class nested inside a class.** `testNestingThroughClass` covers a class inside a *function*. Nothing covers a class inside a class, where the inner class body cannot see the outer class body's names (`class Outer: v = 1; class Inner: w = v` → `NameError`) but *can* see an enclosing function's variable two classes deep (`w = v` with `v` a function local → works). The asymmetry between those two cases is exactly the kind of thing this file exists to pin, and it is absent.

**4.3 `__class__` / zero-argument `super()`.** The implicit `__class__` cell that `super()` closes over is a scope mechanism (a free variable injected by the compiler into methods) and gets no coverage at all — not in nested functions inside methods, not in class bodies, not in `__classcell__` handling with a custom `__prepare__` namespace.

**4.4 PEP 667 / `locals()` write-through.** The file passes identically on 3.11 and 3.14 even though `exec`-to-locals visibility changed in 3.13 (§3.5). There is no test asserting either the old or the new behaviour; `testLocalsFunction` and `testLocalsClass` happen to be insensitive to the difference. If you are porting interpreter internals, this file will not warn you.

**4.5 `testFreeingCell` is dead code.** Lines 651–656:
```python
class Special:
    def __del__(self):
        nestedcell_get()      # undefined name
```
No `Special()` is ever instantiated, so `__del__` never runs, `nestedcell_get` is never resolved, and the test asserts nothing. It passes for the trivial reason that defining a class cannot fail. It appears to be a remnant of a test that was meant to exercise a finaliser touching a cell mid-collection.

**4.6 `testInteractionWithTraceFunc`'s assertion is unrelated to its setup.** It installs a tracer, walks an `AV` code path, uninstalls, then asserts `TypeError` from calling `sys.settrace` with no arguments. Nothing asserts that the trace round-trip preserved scoping.

**4.7 Async scopes.** No `async def`, no `async for`/`await` inside closures or comprehensions, no `contextvars`. `testNonLocalGenerator` covers sync generators only. Async comprehensions have their own scoping rules (they cannot contain `yield`, they capture differently) and are untested here.

**4.8 `except ... as e` target deletion.** The implicit `del e` at the end of an `except` block is a scope rule (the name is unbound afterwards, and `e` is a cell if closed over). No coverage.

**4.9 `del` on globals from a function.** `testUnboundLocal_AfterDel` covers `del` of a *local/cell*. `global x; del x` (deleting a module binding from inside a function) is untested, as is `del` of a name that a closure has already captured but never called.

**4.10 Module-level `nonlocal` and the `nonlocal`-needs-a-binding rule.** The `SyntaxError: no binding for nonlocal 'x' found` path is never asserted, even though it is the rule that makes `nonlocal` unusable at module level. Same for `nonlocal` naming a global.

**4.11 Metaclass namespaces.** `testClassNamespaceOverridesClosure` uses the default dict-backed class namespace. `__prepare__` returning a non-dict mapping (which changes what `LOAD_NAME` does mid-body) is untested.

**4.12 Star-import at module level.** `testUnoptimizedNamespaces` only asserts the four `SyntaxError` cases. The runtime behaviour that motivated the original "unoptimized namespaces" work — a module-level `import *` adding names to `globals()` that a nested function can then resolve as globals — is not exercised. The `if 1:` wrappers around every snippet are vestigial from when these cases were `exec`'d as strings.

**4.13 No free-threading / per-interpreter coverage.** Nothing here distinguishes a shared-object-dict build from a per-object one; `testCellLeak` and `testLeaks` use `gc_collect()` with a comment aimed at PyPy, but no test asserts anything under `--disable-gil`.

---

### Appendix — probe evidence (verbatim excerpts)

```
PY 3.11.9 / 3.14.9
nested-class test(): var | method_and_var(): method | actual_global(): global
class-ns-overrides-closure y: 43
after del x in class body: hasattr(X2,'x')= False global x= 42
class-body-global: get before= 13 after set= 13 module x= 13
C.x = 12 | C.m() = 1
locals() of class body: ['__module__', '__qualname__', 'y', 'm']
unbound-in-outer  -> UnboundLocalError: cannot access local variable 'y' where it is not associated with a value
unbound-in-inner  -> NameError: cannot access free variable 'y' where it is not associated with a value in enclosing scope
eval with bare x ref : 5
eval without bare x  : NameError name 'x' is not defined
eval code-with-freevars: TypeError: code object passed to eval() may not contain free variables
exec code-with-freevars: TypeError: code object requires a closure of exactly length 1
exec-new-name-in-locals():  True (3.11) / False (3.14)
exec-rebind-fast-local: 1   (both)
locals-snapshot: (1, 2)     (both)
import* in func / nested func / class body: SyntaxError: import * only allowed at module level
param-and-global: SyntaxError: name 'a' is parameter and global
nonlocal-at-module-class: SyntaxError: no binding for nonlocal 'q' found
A plain module exec       STOREs: ['STORE_NAME:y','STORE_NAME:zz']           globals: []    locals: ['y','zz']
B with nested global y    STOREs: ['STORE_NAME:f','STORE_GLOBAL:y',...]      globals: ['y'] locals: ['f','zz']
unittest: Ran 40 tests — OK  (3.11.9 and 3.14.9)
```
