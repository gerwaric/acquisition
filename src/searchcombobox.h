#pragma once

#include <QCompleter>
#include <QComboBox>
#include <QProxyStyle>
#include <QTimer>

class QAbstractItemModel;

class SearchComboCompleter : public QCompleter {
    Q_OBJECT
public:
    using QCompleter::QCompleter;
public slots:
    void complete(const QRect& rect = QRect());
};

class SearchComboStyle : public QProxyStyle {
public:
    using QProxyStyle::QProxyStyle;
    int styleHint(StyleHint hint, const QStyleOption* option = nullptr, const QWidget* widget = nullptr, QStyleHintReturn* returnData = nullptr) const override;
private:
    const int TOOLTIP_DELAY_MSEC = 50;
};

class SearchComboBox : public QComboBox {
    Q_OBJECT
public:
    SearchComboBox(QAbstractItemModel* model, QWidget* parent = nullptr);
private slots:
    void OnTextEdited();
    void OnEditTimeout();
    void OnCompleterActivated(const QString& text);
private:
    SearchComboCompleter completer_;
    QTimer edit_timer_;
};
