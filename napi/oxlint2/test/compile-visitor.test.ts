import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
// TODO(camc314): we need to generate `.d.ts` file for this module.
// @ts-expect-error
import { NODE_TYPE_IDS_MAP } from '../../parser/generated/lazy/types.js';
import {
  addVisitorToCompiled,
  compiledVisitor,
  finalizeCompiledVisitor,
  initCompiledVisitor,
} from '../src-js/visitor.js';

import type { EnterExit, VisitFn } from '../src-js/types.ts';

const PROGRAM_TYPE_ID = NODE_TYPE_IDS_MAP.get('Program'),
  EMPTY_STMT_TYPE_ID = NODE_TYPE_IDS_MAP.get('EmptyStatement');

describe('compile visitor', () => {
  beforeEach(initCompiledVisitor);
  afterEach(finalizeCompiledVisitor);

  it('throws if visitor is not an object', () => {
    const expectedErr = new TypeError('Visitor returned from `create` method must be an object');
    expect(() => (addVisitorToCompiled as any)()).toThrow(expectedErr);
    expect(() => addVisitorToCompiled(null)).toThrow(expectedErr);
    expect(() => addVisitorToCompiled(undefined)).toThrow(expectedErr);
    expect(() => addVisitorToCompiled(true as any)).toThrow(expectedErr);
    expect(() => addVisitorToCompiled('xyz' as any)).toThrow(expectedErr);
  });

  it('throws if unknown visitor key', () => {
    expect(() => addVisitorToCompiled({ Foo() {} }))
      .toThrow(new Error("Unknown node type 'Foo' in visitor object"));
  });

  it('throws if visitor property is not a function', () => {
    const expectedErr = new TypeError("'Program' property of visitor object is not a function");
    expect(() => addVisitorToCompiled({ Program: null })).toThrow(expectedErr);
    expect(() => addVisitorToCompiled({ Program: undefined })).toThrow(expectedErr);
    expect(() => addVisitorToCompiled({ Program: true } as any)).toThrow(expectedErr);
    expect(() => addVisitorToCompiled({ Program: {} } as any)).toThrow(expectedErr);
  });

  describe('registers enter visitor', () => {
    it('for leaf node', () => {
      const enter = () => {};
      addVisitorToCompiled({ EmptyStatement: enter });
      expect(finalizeCompiledVisitor()).toBe(true);
      expect(compiledVisitor[EMPTY_STMT_TYPE_ID]).toBe(enter);
    });

    it('for non-leaf node', () => {
      const enter = () => {};
      addVisitorToCompiled({ Program: enter });
      expect(finalizeCompiledVisitor()).toBe(true);

      const enterExit = compiledVisitor[PROGRAM_TYPE_ID] as EnterExit;
      expect(enterExit).toEqual({ enter, exit: null });
      expect(enterExit.enter).toBe(enter);
    });
  });

  describe('registers exit visitor', () => {
    it('for leaf node', () => {
      const exit = () => {};
      addVisitorToCompiled({ 'EmptyStatement:exit': exit });
      expect(finalizeCompiledVisitor()).toBe(true);
      expect(compiledVisitor[EMPTY_STMT_TYPE_ID]).toBe(exit);
    });

    it('for non-leaf node', () => {
      const exit = () => {};
      addVisitorToCompiled({ 'Program:exit': exit });
      expect(finalizeCompiledVisitor()).toBe(true);

      const enterExit = compiledVisitor[PROGRAM_TYPE_ID] as EnterExit;
      expect(enterExit).toEqual({ enter: null, exit });
      expect(enterExit.exit).toBe(exit);
    });
  });

  describe('registers enter and exit visitors', () => {
    describe('for leaf node', () => {
      it('defined in order', () => {
        const enter = vi.fn(() => {});
        const exit = vi.fn(() => {});
        addVisitorToCompiled({ EmptyStatement: enter, 'EmptyStatement:exit': exit });
        expect(finalizeCompiledVisitor()).toBe(true);

        const node = {};
        (compiledVisitor[EMPTY_STMT_TYPE_ID] as VisitFn)(node);
        expect(enter).toHaveBeenCalledWith(node);
        expect(exit).toHaveBeenCalledWith(node);
        expect(enter).toHaveBeenCalledBefore(exit);
      });

      it('defined in reverse order', () => {
        const enter = vi.fn(() => {});
        const exit = vi.fn(() => {});
        addVisitorToCompiled({ 'EmptyStatement:exit': exit, EmptyStatement: enter });
        expect(finalizeCompiledVisitor()).toBe(true);

        const node = {};
        (compiledVisitor[EMPTY_STMT_TYPE_ID] as VisitFn)(node);
        expect(enter).toHaveBeenCalledWith(node);
        expect(exit).toHaveBeenCalledWith(node);
        expect(enter).toHaveBeenCalledBefore(exit);
      });
    });

    describe('for non-leaf node', () => {
      it('defined in order', () => {
        const enter = () => {};
        const exit = () => {};
        addVisitorToCompiled({ Program: enter, 'Program:exit': exit });
        expect(finalizeCompiledVisitor()).toBe(true);

        const enterExit = compiledVisitor[PROGRAM_TYPE_ID] as EnterExit;
        expect(enterExit).toEqual({ enter, exit });
        expect(enterExit.enter).toBe(enter);
        expect(enterExit.exit).toBe(exit);
      });

      it('defined in reverse order', () => {
        const enter = () => {};
        const exit = () => {};
        addVisitorToCompiled({ 'Program:exit': exit, Program: enter });
        expect(finalizeCompiledVisitor()).toBe(true);

        const enterExit = compiledVisitor[PROGRAM_TYPE_ID] as EnterExit;
        expect(enterExit).toEqual({ enter, exit });
        expect(enterExit.enter).toBe(enter);
        expect(enterExit.exit).toBe(exit);
      });
    });
  });

  describe('combines multiple visitors', () => {
    describe('for leaf node', () => {
      it('defined in order', () => {
        const enter1 = vi.fn(() => {});
        const exit1 = vi.fn(() => {});
        addVisitorToCompiled({ EmptyStatement: enter1, 'EmptyStatement:exit': exit1 });

        const enter2 = vi.fn(() => {});
        const exit2 = vi.fn(() => {});
        addVisitorToCompiled({ EmptyStatement: enter2, 'EmptyStatement:exit': exit2 });

        expect(finalizeCompiledVisitor()).toBe(true);

        const node = {};
        (compiledVisitor[EMPTY_STMT_TYPE_ID] as VisitFn)(node);
        expect(enter1).toHaveBeenCalledWith(node);
        expect(exit1).toHaveBeenCalledWith(node);
        expect(enter1).toHaveBeenCalledBefore(exit1);

        expect(enter2).toHaveBeenCalledWith(node);
        expect(exit2).toHaveBeenCalledWith(node);
        expect(enter2).toHaveBeenCalledBefore(exit2);
      });

      it('defined in reverse order', () => {
        const enter1 = vi.fn(() => {});
        const exit1 = vi.fn(() => {});
        addVisitorToCompiled({ 'EmptyStatement:exit': exit1, EmptyStatement: enter1 });

        const enter2 = vi.fn(() => {});
        const exit2 = vi.fn(() => {});
        addVisitorToCompiled({ 'EmptyStatement:exit': exit2, EmptyStatement: enter2 });

        expect(finalizeCompiledVisitor()).toBe(true);

        const node = {};
        (compiledVisitor[EMPTY_STMT_TYPE_ID] as VisitFn)(node);
        expect(enter1).toHaveBeenCalledWith(node);
        expect(exit1).toHaveBeenCalledWith(node);
        expect(enter1).toHaveBeenCalledBefore(exit1);

        expect(enter2).toHaveBeenCalledWith(node);
        expect(exit2).toHaveBeenCalledWith(node);
        expect(enter2).toHaveBeenCalledBefore(exit2);
      });

      it('defined in mixed order', () => {
        const enter1 = vi.fn(() => {});
        const exit1 = vi.fn(() => {});
        addVisitorToCompiled({ EmptyStatement: enter1, 'EmptyStatement:exit': exit1 });

        const enter2 = vi.fn(() => {});
        const exit2 = vi.fn(() => {});
        addVisitorToCompiled({ 'EmptyStatement:exit': exit2, EmptyStatement: enter2 });

        expect(finalizeCompiledVisitor()).toBe(true);

        const node = {};
        (compiledVisitor[EMPTY_STMT_TYPE_ID] as VisitFn)(node);
        expect(enter1).toHaveBeenCalledWith(node);
        expect(exit1).toHaveBeenCalledWith(node);
        expect(enter1).toHaveBeenCalledBefore(exit1);

        expect(enter2).toHaveBeenCalledWith(node);
        expect(exit2).toHaveBeenCalledWith(node);
        expect(enter2).toHaveBeenCalledBefore(exit2);
      });

      it('defined in mixed order 2', () => {
        const enter1 = vi.fn(() => {});
        const exit1 = vi.fn(() => {});
        addVisitorToCompiled({ 'EmptyStatement:exit': exit1, EmptyStatement: enter1 });

        const enter2 = vi.fn(() => {});
        const exit2 = vi.fn(() => {});
        addVisitorToCompiled({ EmptyStatement: enter2, 'EmptyStatement:exit': exit2 });

        expect(finalizeCompiledVisitor()).toBe(true);

        const node = {};
        (compiledVisitor[EMPTY_STMT_TYPE_ID] as VisitFn)(node);
        expect(enter1).toHaveBeenCalledWith(node);
        expect(exit1).toHaveBeenCalledWith(node);
        expect(enter1).toHaveBeenCalledBefore(exit1);

        expect(enter2).toHaveBeenCalledWith(node);
        expect(exit2).toHaveBeenCalledWith(node);
        expect(enter2).toHaveBeenCalledBefore(exit2);
      });
    });

    describe('for non-leaf node', () => {
      it('defined in order', () => {
        const enter1 = vi.fn(() => {});
        const exit1 = vi.fn(() => {});
        addVisitorToCompiled({ Program: enter1, 'Program:exit': exit1 });

        const enter2 = vi.fn(() => {});
        const exit2 = vi.fn(() => {});
        addVisitorToCompiled({ Program: enter2, 'Program:exit': exit2 });

        expect(finalizeCompiledVisitor()).toBe(true);

        const enterExit = compiledVisitor[PROGRAM_TYPE_ID] as EnterExit;

        const enterNode = {};
        enterExit.enter(enterNode);
        expect(enter1).toHaveBeenCalledWith(enterNode);
        expect(enter2).toHaveBeenCalledWith(enterNode);

        const exitNode = {};
        enterExit.exit(exitNode);
        expect(exit1).toHaveBeenCalledWith(exitNode);
        expect(exit2).toHaveBeenCalledWith(exitNode);
      });

      it('defined in reverse order', () => {
        const enter1 = vi.fn(() => {});
        const exit1 = vi.fn(() => {});
        addVisitorToCompiled({ 'Program:exit': exit1, Program: enter1 });

        const enter2 = vi.fn(() => {});
        const exit2 = vi.fn(() => {});
        addVisitorToCompiled({ 'Program:exit': exit2, Program: enter2 });

        expect(finalizeCompiledVisitor()).toBe(true);

        const enterExit = compiledVisitor[PROGRAM_TYPE_ID] as EnterExit;

        const enterNode = {};
        enterExit.enter(enterNode);
        expect(enter1).toHaveBeenCalledWith(enterNode);
        expect(enter2).toHaveBeenCalledWith(enterNode);

        const exitNode = {};
        enterExit.exit(exitNode);
        expect(exit1).toHaveBeenCalledWith(exitNode);
        expect(exit2).toHaveBeenCalledWith(exitNode);
      });

      it('defined in mixed order', () => {
        const enter1 = vi.fn(() => {});
        const exit1 = vi.fn(() => {});
        addVisitorToCompiled({ Program: enter1, 'Program:exit': exit1 });

        const enter2 = vi.fn(() => {});
        const exit2 = vi.fn(() => {});
        addVisitorToCompiled({ 'Program:exit': exit2, Program: enter2 });

        expect(finalizeCompiledVisitor()).toBe(true);

        const enterExit = compiledVisitor[PROGRAM_TYPE_ID] as EnterExit;

        const enterNode = {};
        enterExit.enter(enterNode);
        expect(enter1).toHaveBeenCalledWith(enterNode);
        expect(enter2).toHaveBeenCalledWith(enterNode);

        const exitNode = {};
        enterExit.exit(exitNode);
        expect(exit1).toHaveBeenCalledWith(exitNode);
        expect(exit2).toHaveBeenCalledWith(exitNode);
      });

      it('defined in mixed order 2', () => {
        const enter1 = vi.fn(() => {});
        const exit1 = vi.fn(() => {});
        addVisitorToCompiled({ 'Program:exit': exit1, Program: enter1 });

        const enter2 = vi.fn(() => {});
        const exit2 = vi.fn(() => {});
        addVisitorToCompiled({ Program: enter2, 'Program:exit': exit2 });

        expect(finalizeCompiledVisitor()).toBe(true);

        const enterExit = compiledVisitor[PROGRAM_TYPE_ID] as EnterExit;

        const enterNode = {};
        enterExit.enter(enterNode);
        expect(enter1).toHaveBeenCalledWith(enterNode);
        expect(enter2).toHaveBeenCalledWith(enterNode);

        const exitNode = {};
        enterExit.exit(exitNode);
        expect(exit1).toHaveBeenCalledWith(exitNode);
        expect(exit2).toHaveBeenCalledWith(exitNode);
      });
    });

    it('many visitors', () => {
      const enter1 = vi.fn(() => {});
      const exit1 = vi.fn(() => {});
      addVisitorToCompiled({ EmptyStatement: enter1, 'EmptyStatement:exit': exit1 });

      const enter2 = vi.fn(() => {});
      const exit2 = vi.fn(() => {});
      addVisitorToCompiled({ EmptyStatement: enter2, 'EmptyStatement:exit': exit2 });

      const enter3 = vi.fn(() => {});
      const exit3 = vi.fn(() => {});
      addVisitorToCompiled({ EmptyStatement: enter3, 'EmptyStatement:exit': exit3 });

      const enter4 = vi.fn(() => {});
      const exit4 = vi.fn(() => {});
      addVisitorToCompiled({ EmptyStatement: enter4, 'EmptyStatement:exit': exit4 });

      const enter5 = vi.fn(() => {});
      const exit5 = vi.fn(() => {});
      addVisitorToCompiled({ EmptyStatement: enter5, 'EmptyStatement:exit': exit5 });

      const enter6 = vi.fn(() => {});
      const exit6 = vi.fn(() => {});
      addVisitorToCompiled({ EmptyStatement: enter6, 'EmptyStatement:exit': exit6 });

      expect(finalizeCompiledVisitor()).toBe(true);

      const node = {};
      (compiledVisitor[EMPTY_STMT_TYPE_ID] as VisitFn)(node);

      expect(enter1).toHaveBeenCalledWith(node);
      expect(exit1).toHaveBeenCalledWith(node);
      expect(enter1).toHaveBeenCalledBefore(exit1);

      expect(enter2).toHaveBeenCalledWith(node);
      expect(exit2).toHaveBeenCalledWith(node);
      expect(enter2).toHaveBeenCalledBefore(exit2);

      expect(enter3).toHaveBeenCalledWith(node);
      expect(exit3).toHaveBeenCalledWith(node);
      expect(enter3).toHaveBeenCalledBefore(exit3);

      expect(enter4).toHaveBeenCalledWith(node);
      expect(exit4).toHaveBeenCalledWith(node);
      expect(enter4).toHaveBeenCalledBefore(exit4);

      expect(enter5).toHaveBeenCalledWith(node);
      expect(exit5).toHaveBeenCalledWith(node);
      expect(enter5).toHaveBeenCalledBefore(exit5);

      expect(enter6).toHaveBeenCalledWith(node);
      expect(exit6).toHaveBeenCalledWith(node);
      expect(enter6).toHaveBeenCalledBefore(exit6);
    });
  });

  describe('`finalizeCompiledVisitor` returns false if all visitors empty', () => {
    it('no visitors', () => {
      expect(finalizeCompiledVisitor()).toBe(false);
    });

    it('1 visitor', () => {
      addVisitorToCompiled({});
      expect(finalizeCompiledVisitor()).toBe(false);
    });

    it('multiple visitors', () => {
      addVisitorToCompiled({});
      addVisitorToCompiled({});
      addVisitorToCompiled({});
      expect(finalizeCompiledVisitor()).toBe(false);
    });
  });
});
