from abc import ABCMeta

from typeguard import typechecked


class TypeCheckedABCMeta(ABCMeta):
    def __new__(cls, name, bases, namespace):
        created_cls = super().__new__(cls, name, bases, namespace)
        created_cls = typechecked(created_cls)  # Apply @typechecked to every subclass
        return created_cls
