#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

from abc import abstractmethod
import bisect
from collections.abc import Mapping, MutableMapping
from typing import Any, Generator, TypeVar, Protocol


class Comparable(Protocol):
    @abstractmethod
    def __lt__(self, other: Any, /) -> bool: ...

KeyType = TypeVar("KeyType", bound=Comparable)
ValueType = TypeVar("ValueType")
_OtherType = TypeVar("_OtherType")


class SortedDict(MutableMapping[KeyType, ValueType]):
    def __init__(self, other: Mapping[KeyType, ValueType]|None=None):
        self.map: dict[KeyType, ValueType] = dict()
        self.sortedKeys: list[KeyType] = list()

        if other is not None:
            for key, value in other.items():
                self.add(key, value)


    def add(self, key: KeyType, value: ValueType) -> None:
        if key not in self.map:
            # Avoid adding the key twice if it is already on the map
            bisect.insort(self.sortedKeys, key)
        self.map[key] = value

    def remove(self, key: KeyType) -> None:
        del self.map[key]
        self.sortedKeys.remove(key)

    def get(self, key: KeyType, default: _OtherType=None) -> ValueType | _OtherType:
        return self.map.get(key, default)


    def getKeyRight(self, key: KeyType, inclusive: bool=True) -> tuple[KeyType, ValueType]|None:
        """Returns the pair with the greatest key which is less or equal to the `key` parameter, or None if there's no smaller pair than the passed `key`.

        If `inclusive` is `False`, then the returned pair will be strictly less than the passed `key`.
        """
        if inclusive:
            index = bisect.bisect_right(self.sortedKeys, key)
        else:
            index = bisect.bisect_left(self.sortedKeys, key)
        if index == 0:
            return None
        currentKey = self.sortedKeys[index - 1]
        return currentKey, self.map[currentKey]

    def getKeyLeft(self, key: KeyType, inclusive: bool=True) -> tuple[KeyType, ValueType]|None:
        """Returns the pair with the smallest key which is gretest or equal to the `key` parameter, or None if there's no greater pair than the passed `key`.

        If `inclusive` is `False`, then the returned pair will be strictly greater than the passed `key`.
        """
        if inclusive:
            index = bisect.bisect_left(self.sortedKeys, key)
        else:
            index = bisect.bisect_right(self.sortedKeys, key)
        if index == len(self.sortedKeys):
            return None
        key = self.sortedKeys[index]
        return key, self.map[key]


    def getRange(self, startKey: KeyType, endKey: KeyType, startInclusive: bool=True, endInclusive: bool=False) -> Generator[tuple[KeyType, ValueType], None, None]:
        """Generator which iterates in the range [`startKey`, `endKey`], returining a (key, value) tuple.

        By default the `startKey` is inclusive but the `endKey` isn't, this can be changed with the `startInclusive` and `endInclusive` parameters"""
        if startInclusive:
            keyIndexStart = bisect.bisect_left(self.sortedKeys, startKey)
        else:
            keyIndexStart = bisect.bisect_right(self.sortedKeys, startKey)

        if endInclusive:
            keyIndexEnd = bisect.bisect_right(self.sortedKeys, endKey)
        else:
            keyIndexEnd = bisect.bisect_left(self.sortedKeys, endKey)

        for index in range(keyIndexStart, keyIndexEnd):
            key = self.sortedKeys[index]
            yield (key, self.map[key])

    def getRangeAndPop(self, startKey: KeyType, endKey: KeyType, startInclusive: bool=True, endInclusive: bool=False) -> Generator[tuple[KeyType, ValueType], None, None]:
        """Similar to `getRange`, but every pair is removed from the dictionary.

        Please note this generator iterates in reverse/descending order"""
        if startInclusive:
            keyIndexStart = bisect.bisect_left(self.sortedKeys, startKey)
        else:
            keyIndexStart = bisect.bisect_right(self.sortedKeys, startKey)

        if endInclusive:
            keyIndexEnd = bisect.bisect_right(self.sortedKeys, endKey)
        else:
            keyIndexEnd = bisect.bisect_left(self.sortedKeys, endKey)

        for index in range(keyIndexEnd-1, keyIndexStart-1, -1):
            key = self.sortedKeys[index]
            value = self.map[key]
            self.remove(key)
            yield (key, value)

    def pop(self, key: KeyType, default: ValueType|_OtherType=None) -> ValueType|_OtherType:
        if key not in self.map:
            return default
        value = self.map[key]
        self.remove(key)
        return value

    def __getitem__(self, key: KeyType) -> ValueType:
        return self.map[key]

    def __setitem__(self, key: KeyType, value: ValueType) -> None:
        self.add(key, value)

    def __delitem__(self, key: KeyType) -> None:
        self.remove(key)

    def __iter__(self) -> Generator[KeyType, None, None]:
        "Iteration is sorted by keys"
        for key in self.sortedKeys:
            yield key

    def __len__(self) -> int:
        return len(self.map)

    def __contains__(self, key: KeyType) -> bool:
        return self.map.__contains__(key)


    def __str__(self) -> str:
        ret = "SortedDict({"
        comma = False
        for key, value in self.items():
            if comma:
                ret += ", "
            ret += f"{repr(key)}: {repr(value)}"
            comma = True
        ret += "})"
        return ret

    def __repr__(self) -> str:
        return self.__str__()
