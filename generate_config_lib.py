#!/usr/bin/env nix-shell
#!nix-shell -i python3 -p "python310.withPackages(ps: with ps; [pyyaml])"

from __future__ import annotations
from dataclasses import dataclass
from typing import Callable
import yaml


# provides calendar type and modifications


@dataclass
class CalendarConfig:
    inner: CalendarConfigSum

    def modify(self: CalendarConfig, modifications: list[ComponentModification]):
        match self.inner:
            case MergeCalendars(_):
                self.inner = ModifyCalendar(self.inner, modifications)
            case ModifyCalendar(calendar, _):
                new_modifications = self.inner.modifications + modifications
                self.inner = ModifyCalendar(calendar, new_modifications)
            case FetchCalendar(_):
                self.inner = ModifyCalendar(self.inner, modifications)

    def keep_properties_if_name_in(self: CalendarConfig, names: list[str]):
        self.modify([KeepPropertiesIfNameIn(names)])

    def remove_properties_if_name_in(self: CalendarConfig, names: list[str]):
        self.modify([RemovePropertiesIfNameIn(names)])

    def replace_properties_value_if_name_is(
        self: CalendarConfig, name: str, value: str
    ):
        self.modify([ReplacePropertiesValueIfNameIs(name, value)])

    def insert_property(self: CalendarConfig, property: str):
        self.modify([InsertProperty(property)])

    def keep_components_if_name_in(self: CalendarConfig, names: list[str]):
        self.modify([KeepComponentsIfNameIn(names)])

    def remove_components_if_name_in(self: CalendarConfig, names: list[str]):
        self.modify([RemoveComponentsIfNameIn(names)])

    def modify_components_if_name_is(
        self: CalendarConfig,
        name: str,
        modifications: Callable[[CalendarConfig], None],
    ):
        # create dummy calendar config
        dummy_calendar_config = CalendarConfig(ModifyCalendar(self.inner, []))
        # modify dummy calendar config
        modifications(dummy_calendar_config)
        # return modified calendar config
        assert isinstance(dummy_calendar_config.inner, ModifyCalendar)
        self.modify(
            [ModifyComponentsIfNameIs(name, dummy_calendar_config.inner.modifications)]
        )


def calendar_config_container_representer(dumper: yaml.Dumper, data: CalendarConfig):
    return dumper.represent_data(data.inner)


yaml.add_representer(CalendarConfig, calendar_config_container_representer)


@dataclass
class MergeCalendars:
    calendars: list[CalendarConfigSum]


def merge_calendars(calendars: list[CalendarConfig]):
    return CalendarConfig(MergeCalendars([calendar.inner for calendar in calendars]))


def merge_calendars_representer(dumper: yaml.Dumper, data: MergeCalendars):
    return dumper.represent_dict({"MergeCalendars": data.calendars})


yaml.add_representer(MergeCalendars, merge_calendars_representer)


@dataclass
class ModifyCalendar:
    calendar: CalendarConfigSum
    modifications: list


def modify_calendar(calendar: CalendarConfigSum, modifications: list):
    return CalendarConfig(ModifyCalendar(calendar, modifications))


def modify_calendar_representer(dumper: yaml.Dumper, data: ModifyCalendar):
    return dumper.represent_dict(
        {"ModifyCalendar": [data.calendar, data.modifications]}
    )


yaml.add_representer(ModifyCalendar, modify_calendar_representer)


@dataclass
class FetchCalendar:
    url: str


def fetch_calendar(url: str):
    return CalendarConfig(FetchCalendar(url))


def fetch_calendar_representer(dumper: yaml.Dumper, data: FetchCalendar):
    return dumper.represent_dict({"FetchCalendar": data.url})


yaml.add_representer(FetchCalendar, fetch_calendar_representer)

CalendarConfigSum = MergeCalendars | ModifyCalendar | FetchCalendar


class ComponentModificationBase:
    pass


@dataclass
class KeepPropertiesIfNameIn(ComponentModificationBase):
    names: list[str]


def keep_properties_if_name_in_representer(
    dumper: yaml.Dumper, data: KeepPropertiesIfNameIn
):
    return dumper.represent_dict({"KeepPropertiesIfNameIn": data.names})


yaml.add_representer(KeepPropertiesIfNameIn, keep_properties_if_name_in_representer)


@dataclass
class RemovePropertiesIfNameIn(ComponentModificationBase):
    names: list[str]


def remove_properties_if_name_in_representer(
    dumper: yaml.Dumper, data: RemovePropertiesIfNameIn
):
    return dumper.represent_dict({"RemovePropertiesIfNameIn": data.names})


yaml.add_representer(RemovePropertiesIfNameIn, remove_properties_if_name_in_representer)


@dataclass
class ReplacePropertiesValueIfNameIs(ComponentModificationBase):
    name: str
    value: str


def replace_properties_value_if_name_is_representer(
    dumper: yaml.Dumper, data: ReplacePropertiesValueIfNameIs
):
    return dumper.represent_dict(
        {"ReplacePropertiesValueIfNameIs": [data.name, data.value]}
    )


yaml.add_representer(
    ReplacePropertiesValueIfNameIs, replace_properties_value_if_name_is_representer
)


@dataclass
class InsertProperty(ComponentModificationBase):
    property: str


def insert_property_representer(dumper: yaml.Dumper, data: InsertProperty):
    return dumper.represent_dict({"InsertProperty": data.property})


yaml.add_representer(InsertProperty, insert_property_representer)


@dataclass
class KeepComponentsIfNameIn(ComponentModificationBase):
    names: list[str]


def keep_components_if_name_in_representer(
    dumper: yaml.Dumper, data: KeepComponentsIfNameIn
):
    return dumper.represent_dict({"KeepComponentsIfNameIn": data.names})


yaml.add_representer(KeepComponentsIfNameIn, keep_components_if_name_in_representer)


@dataclass
class RemoveComponentsIfNameIn(ComponentModificationBase):
    names: list[str]


def remove_components_if_name_in_representer(
    dumper: yaml.Dumper, data: RemoveComponentsIfNameIn
):
    return dumper.represent_dict({"RemoveComponentsIfNameIn": data.names})


yaml.add_representer(RemoveComponentsIfNameIn, remove_components_if_name_in_representer)


@dataclass
class ModifyComponentsIfNameIs(ComponentModificationBase):
    name: str
    modifications: list[ComponentModification]


def modify_components_if_name_is_representer(
    dumper: yaml.Dumper, data: ModifyComponentsIfNameIs
):
    return dumper.represent_dict(
        {"ModifyComponentsIfNameIs": [data.name, data.modifications]}
    )


yaml.add_representer(ModifyComponentsIfNameIs, modify_components_if_name_is_representer)

ComponentModification = (
    KeepPropertiesIfNameIn
    | RemovePropertiesIfNameIn
    | ReplacePropertiesValueIfNameIs
    | InsertProperty
    | KeepComponentsIfNameIn
    | RemoveComponentsIfNameIn
    | ModifyComponentsIfNameIs
)
