from typeguard import typechecked


class TypeCheckedMeta(type):
    def __new__(cls, name, bases, namespace):
        created_cls = super().__new__(cls, name, bases, namespace)
        created_cls = typechecked(created_cls)  # Apply @typechecked to every subclass
        return created_cls
