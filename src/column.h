// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2014 Ilya Zhuravlev

#pragma once

#include <QColor>
#include <QString>
#include <QVariant>

#include "item.h"

class BuyoutManager;

class Column
{
public:
    Column() = default;

    // Non-copyable
    Column(const Column &) = delete;
    Column &operator=(const Column &) = delete;

    // Moveable
    Column(Column &&) = default;
    Column &operator=(Column &&) = default;

    virtual QString name() const = 0;
    virtual QVariant value(const Item &item) const = 0;
    virtual QVariant icon(const Item &item) const = 0;
    virtual QColor color(const Item &item) const;
    virtual bool lt(const Item *lhs, const Item *rhs) const;
    virtual ~Column() {}

private:
    typedef std::tuple<double, QString, double, QString, const Item &> sort_tuple;
    sort_tuple multivalue(const Item *item) const;
};

class NameColumn : public Column
{
public:
    QString name() const;
    QVariant value(const Item &item) const;
    QColor color(const Item &item) const;
    QVariant icon(const Item &item) const
    {
        Q_UNUSED(item);
        return QVariant::fromValue(NULL);
    }
};

class CorruptedColumn : public Column
{
public:
    QString name() const;
    QVariant value(const Item &item) const;
    QVariant icon(const Item &item) const
    {
        Q_UNUSED(item);
        return QVariant::fromValue(NULL);
    }
};

class CraftedColumn : public Column
{
public:
    QString name() const;
    QVariant value(const Item &item) const;
    QVariant icon(const Item &item) const
    {
        Q_UNUSED(item);
        return QVariant::fromValue(NULL);
    }
};

class EnchantedColumn : public Column
{
public:
    QString name() const;
    QVariant value(const Item &item) const;
    QVariant icon(const Item &item) const
    {
        Q_UNUSED(item);
        return QVariant::fromValue(NULL);
    }
};

class InfluncedColumn : public Column
{
public:
    QString name() const;
    QVariant value(const Item &item) const;
    QVariant icon(const Item &item) const;
};

// Returns values from item -> properties
class PropertyColumn : public Column
{
public:
    explicit PropertyColumn(const QString &name);
    PropertyColumn(const QString &name, const QString &property);
    QString name() const;
    QVariant value(const Item &item) const;
    QVariant icon(const Item &item) const
    {
        Q_UNUSED(item);
        return QVariant::fromValue(NULL);
    }

private:
    QString m_name;
    QString m_property;
};

class DPSColumn : public Column
{
public:
    QString name() const;
    QVariant value(const Item &item) const;
    QVariant icon(const Item &item) const
    {
        Q_UNUSED(item);
        return QVariant::fromValue(NULL);
    }
};

class pDPSColumn : public Column
{
public:
    QString name() const;
    QVariant value(const Item &item) const;
    QVariant icon(const Item &item) const
    {
        Q_UNUSED(item);
        return QVariant::fromValue(NULL);
    }
};

class eDPSColumn : public Column
{
public:
    QString name() const;
    QVariant value(const Item &item) const;
    QVariant icon(const Item &item) const
    {
        Q_UNUSED(item);
        return QVariant::fromValue(NULL);
    }
};

class ElementalDamageColumn : public Column
{
public:
    explicit ElementalDamageColumn(int index);
    QString name() const;
    QVariant value(const Item &item) const;
    QColor color(const Item &item) const;
    QVariant icon(const Item &item) const
    {
        Q_UNUSED(item);
        return QVariant::fromValue(NULL);
    }

private:
    size_t m_index;
};

class ChaosDamageColumn : public Column
{
public:
    QString name() const;
    QVariant value(const Item &item) const;
    QColor color(const Item &item) const;
    QVariant icon(const Item &item) const
    {
        Q_UNUSED(item);
        return QVariant::fromValue(NULL);
    }
};

class cDPSColumn : public Column
{
public:
    QString name() const;
    QVariant value(const Item &item) const;
    QVariant icon(const Item &item) const
    {
        Q_UNUSED(item);
        return QVariant::fromValue(NULL);
    }
};

class PriceColumn : public Column
{
public:
    explicit PriceColumn(const BuyoutManager &bo_manager);
    QString name() const;
    QVariant value(const Item &item) const;
    QColor color(const Item &item) const;
    bool lt(const Item *lhs, const Item *rhs) const;
    QVariant icon(const Item &item) const
    {
        Q_UNUSED(item);
        return QVariant::fromValue(NULL);
    }

private:
    std::tuple<int, double, const Item &> multivalue(const Item *item) const;
    const BuyoutManager &m_bo_manager;
};

class DateColumn : public Column
{
public:
    explicit DateColumn(const BuyoutManager &bo_manager);
    QString name() const;
    QVariant value(const Item &item) const;
    bool lt(const Item *lhs, const Item *rhs) const;
    QVariant icon(const Item &item) const
    {
        Q_UNUSED(item);
        return QVariant::fromValue(NULL);
    }

private:
    const BuyoutManager &m_bo_manager;
};

class ItemlevelColumn : public Column
{
public:
    QString name() const;
    QVariant value(const Item &item) const;
    QVariant icon(const Item &item) const
    {
        Q_UNUSED(item);
        return QVariant::fromValue(NULL);
    }
};
