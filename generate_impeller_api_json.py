#!/bin/python
import json
import os
import sys
import re
import cffi
from logging import info, error, warning
from enum import Enum
from typing import Dict, List, Any


class ImpellerAPI:
    def __init__(self, header_without_comments: str) -> None:
        self.enums: Dict[str, List[str]] = parse_enums_from_header(header_without_comments)
        self.handles: Dict[str, str] = parses_handles_from_header(header_without_comments)
        self.functions: Dict[str, Dict[str, Any]] = parse_functions_from_header(header_without_comments)
        self.pod_structs: Dict[str, Dict[str, Any]] = parses_structs_from_header(header_without_comments)
        # gather all unique types used in the Impeller API that are not handles, enums or pointers to pod structs.
        types = set()
        for struct in self.pod_structs.values():
            for field in struct['fields']:
                types.add(field['ty'])
        for function in self.functions.values():
            for arg in function['args']:
                types.add(arg['ty'])
            types.add(function['return_ty']['ty'])
        for handle in self.handles.keys():
            types.discard(handle)
        for enum in self.enums.keys():
            types.discard(enum)
        for struct in self.pod_structs.keys():
            types.discard(f'const {struct}*')
            types.discard(f'{struct}*')
            types.discard(f'{struct}')

        self.types = sorted(types)





def read_impeller_header_to_str():
    impeller_header_path = "impeller.h"
    with open(impeller_header_path, "r") as impeller_header:
        return impeller_header.read()


def strip_line_comments_from_header_contents(header_contents: str):
    """
    strips comments to reduce regex complexity when matching other objects.
    does not strip block comments or doc comments (///).
    """
    # [^/] match any character that is not / to skip matching last two / chars of ///
    # // match two slashes
    # (?!/) negative lookahead to skip matching first two / chars of ///
    # if negative lookahead succeeds, match any character until end of line (but not newline)
    header_without_comments = re.sub(r"[^/]//(?!/).*", "", header_contents)
    # there's no block comments in the header
    return header_without_comments


def parse_enums_from_header(header_without_comments):
    """
    assumes that there's no line comments in header
    enums don't have doc comments in impeller.h, so we just don't bother parsing doc comments
    we return a list of dictionaries (enum_name: str, enum_values: List[str])
    """
    # captures enum name and its block
    enum_pattern = r"enum\s+(\w+)\s*{([\w\s\d=<,]+)}"
    enums = re.findall(enum_pattern, header_without_comments)
    def parse_enum_values(enum_values: str):
        "parses enum values from block"
        variant_pattern = r"(\w+)\s*.*,"
        variants_list = re.findall(variant_pattern, enum_values)
        return [value.strip() for value in variants_list]
    return {enum[0].strip(): parse_enum_values(enum[1]) for enum in enums}

def parses_handles_from_header(header_without_comments):
    """
    assumes that there's no line comments in header.
    parses doc comments for opaque structs too.
    opaque structs (or handles) are defined with IMPELLER_DEFINE_HANDLE macro.
    we return a list of dictionaries (name: str, doc: str) as there's no fields in opaque structs.
    """
    # captures doc comments and opaque struct name
    # (^///.* match any line that starts with /// followed by anything
    #   (?:\n///.*)* repeatedly match (newline + /// + anything) to consume all doc comments
    # )? doc comment is optional
    # \nIMPEL1LER_DEFINE_HANDLE\((\w+)\) captures the opaque struct name after a newline
    opaque_struct_pattern = r"(^///.*(?:\n///.*)*)?\nIMPELLER_DEFINE_HANDLE\((\w+)\);"
    opaque_structs = re.findall(opaque_struct_pattern, header_without_comments, re.MULTILINE)
    return {struct[1].strip(): struct[0].strip() for struct in opaque_structs}

def parses_structs_from_header(header_without_comments) -> Dict[str, Dict[str, Any]]:
    "assumes that there's no comments in header"
    pod_struct_pattern = r"(///.*(?:\n///.*)*)?\ntypedef struct\s+(\w+)\s*{([^}]+)}"
    pod_structs = re.findall(pod_struct_pattern, header_without_comments)
    structs = {}
    for struct_match in pod_structs:
        name = struct_match[1].strip()
        doc = struct_match[0].strip()
        fields = []
        fields_match = struct_match[2].strip()
        if fields_match:
            for field_match in fields_match.split(';'):
                if field_match:
                    field = parse_var_ty_declaration(field_match)
                    fields.append(field)
        structs[name] = {
            "doc": doc,
            "fields": fields
        }
    return structs


def parse_var_ty_declaration(var_ty_declaration: str) -> Dict[str, Any]:
    """
    parses type + identifier pair from a declaration.
    eg: "int a" -> ("int" is the type, "a" is the ident)
    we will also check for nullable qualifiers
    """
    var_ty_pattern = r"(///.*(?:\n///.*)*\s+)?((?:IMPELLER_NULLABLE|IMPELLER_NONNULL))?\s*(.*?)\s+((?:IMPELLER_NULLABLE|IMPELLER_NONNULL))?\s*(\w+)(\[\d+\])?$"
    var_ty_match = re.match(var_ty_pattern, var_ty_declaration.strip())
    if var_ty_match is None:
        raise ValueError(f"var_ty_declaration is not in correct format: {var_ty_declaration}")
    doc = var_ty_match.group(1)
    name = var_ty_match.group(5).strip()
    ty = ty=var_ty_match.group(3).strip()
    nonnull=var_ty_match.group(4) == "IMPELLER_NONNULL"
    array_match = var_ty_match.group(6)
    array_size = int(array_match.strip('[]')) if array_match else 0
    # for cases where the type is a pointer to pointers. eg: ImpellerTexture*
    ty_nonnull = var_ty_match.group(2) == "IMPELLER_NONNULL"
    result = {
        "name": name,
        "ty": ty,
    }
    if doc:
        result["doc"] = doc
    if nonnull:
        result["nonnull"] = nonnull
    if array_size:
        result["array_size"] = array_size
    if ty_nonnull:
        result["ty_nonnull"] = ty_nonnull
    return result
def parse_functions_from_header(header_without_comments) -> Dict[str, Dict[str, Any]]:
    "assumes that there's no comments in header"
    function_pattern = r"(///.*(?:\n///.*)*)?\nIMPELLER_EXPORT\s+(IMPELLER_NODISCARD\s+)?(\w+\s+)(IMPELLER_NULLABLE|IMPELLER_NONNULL)?\s*(\w+\s*)\(([^)]*)\);"
    functions = re.findall(function_pattern, header_without_comments)

    result = {}
    for function in functions:
        name = function[4].strip()
        return_ty = {
            "ty": function[2].strip(),}
        if function[1]:
            return_ty["nodiscard"] = True
        if function[3].strip() == "IMPELLER_NONNULL":
            return_ty["nonnull"] = True
        args = []
        args_match = function[5].strip()
        if args_match:
            for arg_match in args_match.split(','):
                if arg_match:
                    arg = parse_var_ty_declaration(arg_match)
                    args.append(arg)
        result[name] = {
            "return_ty": return_ty,
            "args": args
        }
    return result





def generate_impeller_api(impeller_header_contents: str) -> ImpellerAPI:
    header_without_comments = strip_line_comments_from_header_contents(impeller_header_contents)
    return ImpellerAPI(header_without_comments)



def fake_main():
    impeller_header_contents = read_impeller_header_to_str()
    api_json = generate_impeller_api(impeller_header_contents)
    with open('impeller_api.json', 'w') as f:
        f.write(json.dumps(api_json.__dict__, indent=2, ))

if __name__ == "__main__":
    fake_main()