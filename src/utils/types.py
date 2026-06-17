from dataclasses import dataclass
from typing import NewType

FilePath = NewType("FilePath", str)
DictHeader = NewType("DictHeader", str)


@dataclass(frozen=True)
class CodeText:
	"""编码-文本对，替代tuple[str, str]"""

	code: str
	text: str


@dataclass(frozen=True)
class CodeWeightText:
	"""编码-权重-文本三元组，替代tuple[str, int, str]"""

	code: str
	weight: int
	text: str
