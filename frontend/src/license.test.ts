import {beforeEach,describe,expect,it,vi} from 'vitest';
import {consumeLicenseFromUrl,storedToken,storeToken} from './license';
describe('license handoff',()=>{beforeEach(()=>{localStorage.clear();history.replaceState({},'', '/?license=abc123&view=atlas')});it('stores and strips checkout token',()=>{consumeLicenseFromUrl();expect(storedToken()).toBe('abc123');expect(location.search).toBe('?view=atlas')});it('restores a pasted token',()=>{storeToken('  restored  ');expect(storedToken()).toBe('restored')})});
