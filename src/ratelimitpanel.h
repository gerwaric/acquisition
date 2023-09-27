#pragma once

#include <QObject>
#include <QPushButton>
#include <QTextEdit>

class MainWindow;

namespace Ui {
    class MainWindow;
}

class RateLimitPanel;

class RateLimitPanelSignalHandler : public QObject {
    Q_OBJECT
public:
    RateLimitPanelSignalHandler(RateLimitPanel& parent) :
        parent_(parent)
    {}
public slots:
    void OnStatusLabelClicked();
private:
    RateLimitPanel& parent_;
};

class RateLimitPanel : public QObject {
    Q_OBJECT
    friend class RateLimitPanelSignalHandler;
public:
    RateLimitPanel(MainWindow* window, Ui::MainWindow* ui);
public slots:
    void OnStatusUpdate(const QString message);
private:
    void UpdateStatusLabel();
    void ToggleOutputVisibility();
    QPushButton* status_button_;
    QTextEdit* output_;
    RateLimitPanelSignalHandler signal_handler_;
};
