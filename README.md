# Star Engine

```text
[eframe UI] ←→ [Bevy 백엔드 엔진]
  StarEditor       Sprite/Scene/Transform
     ↑                     ↑
     └─── Mutex / Channel ─┘ ← 오브젝트 정보 교환
```