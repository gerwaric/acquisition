// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2014 Ilya Zhuravlev

#include "filters.h"

#include <QHBoxLayout>
#include <QLabel>
#include <QLineEdit>

#include <algorithm>
#include <memory>

#include "util/util.h"

std::unique_ptr<FilterData> Filter::CreateData()
{
    return std::make_unique<FilterData>(this);
}

Filter::Filter(const FilterCallbacks &callbacks)
    : m_callbacks(callbacks)
{}

FilterData::FilterData(Filter *filter)
    : r(0)
    , g(0)
    , b(0)
    , r_filled(false)
    , g_filled(false)
    , b_filled(false)
    , m_filter(filter)
{}

bool FilterData::Matches(const std::shared_ptr<Item> &item)
{
    return m_filter->Matches(item, this);
}

void FilterData::FromForm()
{
    m_filter->FromForm(this);
}

void FilterData::ToForm()
{
    m_filter->ToForm(this);
}

SocketsColorsFilter::SocketsColorsFilter(QLayout *parent, const FilterCallbacks &callbacks)
    : Filter(callbacks)
{
    Initialize(parent, "Colors", callbacks);
}

// TODO(xyz): ugh, a lot of copypasta below, perhaps this could be done
// in a nice way?
void SocketsColorsFilter::Initialize(QLayout *parent,
                                     const char *caption,
                                     const FilterCallbacks &callbacks)
{
    m_callbacks = callbacks;
    QWidget *group = new QWidget;
    QHBoxLayout *layout = new QHBoxLayout;
    layout->setContentsMargins(0, 0, 0, 0);
    QLabel *label = new QLabel(caption);
    label->setAlignment(Qt::AlignRight | Qt::AlignVCenter);
    m_textbox_r = new QLineEdit;
    m_textbox_r->setPlaceholderText("R");
    m_textbox_g = new QLineEdit;
    m_textbox_g->setPlaceholderText("G");
    m_textbox_b = new QLineEdit;
    m_textbox_b->setPlaceholderText("B");
    layout->addWidget(label);
    layout->addWidget(m_textbox_r);
    layout->addWidget(m_textbox_g);
    layout->addWidget(m_textbox_b);
    group->setLayout(layout);
    parent->addWidget(group);
    m_textbox_r->setFixedWidth(Util::TextWidth(TextWidthId::WIDTH_RGB));
    m_textbox_g->setFixedWidth(Util::TextWidth(TextWidthId::WIDTH_RGB));
    m_textbox_b->setFixedWidth(Util::TextWidth(TextWidthId::WIDTH_RGB));
    label->setFixedWidth(Util::TextWidth(TextWidthId::WIDTH_LABEL));
    QObject::connect(m_textbox_r,
                     &QLineEdit::textEdited,
                     m_callbacks.receiver,
                     m_callbacks.onChanged);
    QObject::connect(m_textbox_g,
                     &QLineEdit::textEdited,
                     m_callbacks.receiver,
                     m_callbacks.onChanged);
    QObject::connect(m_textbox_b,
                     &QLineEdit::textEdited,
                     m_callbacks.receiver,
                     m_callbacks.onChanged);
}

void SocketsColorsFilter::FromForm(FilterData *data)
{
    data->r_filled = m_textbox_r->text().size() > 0;
    data->g_filled = m_textbox_g->text().size() > 0;
    data->b_filled = m_textbox_b->text().size() > 0;
    data->r = m_textbox_r->text().toInt();
    data->g = m_textbox_g->text().toInt();
    data->b = m_textbox_b->text().toInt();
    m_active = data->r_filled || data->g_filled || data->b_filled;
}

void SocketsColorsFilter::ToForm(FilterData *data)
{
    if (data->r_filled) {
        m_textbox_r->setText(QString::number(data->r));
    }
    if (data->g_filled) {
        m_textbox_g->setText(QString::number(data->g));
    }
    if (data->b_filled) {
        m_textbox_b->setText(QString::number(data->b));
    }
}

void SocketsColorsFilter::ResetForm()
{
    m_textbox_r->setText("");
    m_textbox_g->setText("");
    m_textbox_b->setText("");
    m_active = false;
}

bool SocketsColorsFilter::Check(
    int need_r, int need_g, int need_b, int got_r, int got_g, int got_b, int got_w)
{
    int diff = std::max(0, need_r - got_r) + std::max(0, need_g - got_g)
               + std::max(0, need_b - got_b);
    return diff <= got_w;
}

bool SocketsColorsFilter::Matches(const std::shared_ptr<Item> &item, FilterData *data)
{
    if (!data->r_filled && !data->g_filled && !data->b_filled) {
        return true;
    }
    const int need_r = data->r_filled ? data->r : 0;
    const int need_g = data->g_filled ? data->g : 0;
    const int need_b = data->b_filled ? data->b : 0;
    const ItemSocketGroup &sockets = item->sockets();
    return Check(need_r, need_g, need_b, sockets.r, sockets.g, sockets.b, sockets.w);
}

LinksColorsFilter::LinksColorsFilter(QLayout *parent, const FilterCallbacks &callbacks)
{
    Initialize(parent, "Linked", callbacks);
}

bool LinksColorsFilter::Matches(const std::shared_ptr<Item> &item, FilterData *data)
{
    if (!data->r_filled && !data->g_filled && !data->b_filled) {
        return true;
    }
    const int need_r = data->r_filled ? data->r : 0;
    const int need_g = data->g_filled ? data->g : 0;
    const int need_b = data->b_filled ? data->b : 0;
    for (auto &group : item->socket_groups()) {
        if (Check(need_r, need_g, need_b, group.r, group.g, group.b, group.w)) {
            return true;
        }
    }
    return false;
}
