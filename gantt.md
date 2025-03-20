gantt
    title MarkItUp 开发里程碑 🗓️
    dateFormat  Y-MM-DD
    axisFormat %m/%d
    tickInterval 7day

    section 技术预研 🔍
    文档格式规范研究       :crit, active, a1, after m0, 5d
    Rust技术栈选型        :crit, a2, after a1, 5d
    富文本解析工具评估     :a3, 2025-03-25, 7d
    GUI框架原型验证       :a4, after a3, 5d
    LLM接口适配研究       :a5, after a4, 5d

    section 核心引擎 ⚙️
    Word/XML解析器开发    :crit, b1, 2025-04-01, 10d
    Excel/CSV转换模块     :b2, after b1, 8d
    PDF结构化解析器      :crit, b3, 2025-04-12, 12d
    OCR引擎集成调试       :b4, after b3, 10d
    ASR流水线优化         :b5, after b4, 7d
    LLM数据预处理模块     :b6, after b5, 5d

    section 系统集成 🧩
    GUI界面原型开发       :crit, c1, 2025-05-01, 7d
    RESTful API开发       :crit, c2, after c1, 10d
    自动化测试框架搭建     :c3, after c2, 5d

    section 质量保障 🔧
    单元测试覆盖率提升     :d1, 2025-05-20, 10d
    性能基准测试          :crit, d2, after d1, 7d

    section 交付准备 🚀
    用户手册编撰          :e1, 2025-06-01, 5d
    演示系统部署          :crit, e2, after e1, 3d
    答辩准备          :crit, e3, after e2, 2d

    section 关键节点 🎯
    开题报告   :milestone, m1, 2025-03-20, 0d
    架构评审（暂定）   :milestone, m2, 2025-04-15, 0d
    最终答辩   :milestone, m4, 2025-06-05, 0d

