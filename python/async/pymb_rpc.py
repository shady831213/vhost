from typing import Union
from collections.abc import Coroutine
from asyncio import Event, get_event_loop
g_callback = {}
g_running = {}
def register_callback(callback) -> Coroutine:
    event = Event()
    async def wrapper(id, *args):
        coro = callback(id, *args)
        if isinstance(coro, Coroutine):
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
    task = get_event_loop().create_task(g_callback[callback_name](id, *args))
    key = callback_name + "_" + id
    g_running[key] = task
    return True

def poll(callback_name, id, *args):
    key = callback_name + "_" + id
    result = 0
    if key in g_running:
        finished = g_running[key].done()
        if finished:
            result = g_running[key].result()
            del g_running[key]
        return (True, finished, result)
    else :
        if not _call_callback(callback_name, id, *args):
            return (False, False, result)
        return poll(callback_name, id, *args)
