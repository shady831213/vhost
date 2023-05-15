import cocotb
from typing import Union
from cocotb.decorators import RunningTask
from collections.abc import Coroutine
from cocotb.triggers import Event
g_dut = None
g_callback = {}
g_running = {}
def set_dut(dut):
    global g_dut
    g_dut = dut

def get_dut():
    return g_dut

def register_callback(callback) -> Coroutine:
    event = Event()
    async def wrapper(id, *args):
        coro = callback(g_dut, id, *args)
        if isinstance(coro, RunningTask):
            if isinstance(coro._coro, Coroutine):
                ret = await coro._coro
            else:
                raise TypeError("register_callback: the decorated callback {callback.__name__} by @cocotb.coroutine must be a Coroutine!")
        elif isinstance(coro, Coroutine):
            ret = await coro
        else:
            raise TypeError(f"register_callback: callback {callback.__name__} must be a Coroutine, but it is {type(callback)}!")
        event.set()
        return ret
    g_callback[callback.__name__] = wrapper

    async def waiter():
        await event.wait()
        event.clear()

    return waiter

def _call_callback(callback_name, id, *args):
    if not callback_name in g_callback:
        return False
    task = cocotb.fork(g_callback[callback_name](id, *args))
    key = callback_name + "_" + id
    g_running[key] = task
    return True

def poll(callback_name, id, *args):
    key = callback_name + "_" + id
    result = 0
    if key in g_running:
        finished = not g_running[key]
        if finished:
            result = g_running[key].retval
            del g_running[key]
        return (True, finished, result)
    else :
        if not _call_callback(callback_name, id, *args):
            return (False, False, result)
        return poll(callback_name, id, *args)
